mod support;

use chrono::NaiveDateTime;
use support::{SampleWorkspace, TestDay, TimedWorkspace};
use tiki_core::activity::ItemEvent;
use tiki_core::agenda::AgendaEntry;
use tiki_core::capture::ConfirmedCapture;
use tiki_core::domain::{ItemKind, Priority, Repeat, Task, TaskId, TaskStatus};
use tiki_core::items::ItemService;
use tiki_core::mood::Mood;
use tiki_core::storage::Database;
use tiki_core::workspace::{FixedClock, IslandSnapshot, TikiWorkspace};
use tiki_core::{TikiError, TikiSettings};

#[test]
fn an_item_goes_to_the_trash_once_and_comes_back_once() {
    let mut task = Task::new("Informe").unwrap();
    let nine = TestDay::today().and_hms_opt(9, 0, 0).unwrap();

    assert_eq!(task.trash(nine), Some(ItemEvent::Trashed));
    assert_eq!(task.trashed_at, Some(nine));
    assert_eq!(task.trash(nine), None);

    assert_eq!(task.restore(), Some(ItemEvent::Restored));
    assert!(!task.is_trashed());
    assert_eq!(task.restore(), None);
}

#[test]
fn the_trash_moment_is_saved_with_the_item() {
    let sample = SampleWorkspace::create();
    let tasks = sample.database.tasks();
    let nine = TestDay::today().and_hms_opt(9, 0, 0).unwrap();
    let task_id = tasks
        .insert(&Task::new("Informe").unwrap(), TestDay::start())
        .unwrap();
    let mut task = tasks.find(task_id).unwrap();
    assert_eq!(task.trashed_at, None);

    task.trash(nine);
    tasks.update(&task, nine).unwrap();

    assert_eq!(tasks.find(task_id).unwrap().trashed_at, Some(nine));
}

fn reminder_at(title: &str, at: NaiveDateTime) -> ConfirmedCapture {
    ConfirmedCapture {
        title: title.into(),
        project_name: Some("Altamira".into()),
        due_date: None,
        priority: Priority::Normal,
        remind_at: Some(at),
        kind: ItemKind::Reminder,
        tags: vec!["llamadas".into()],
        repeat: Repeat::Never,
        waiting_for: None,
    }
}

/// The sample item with this title, found in the Tasks tab.
fn id_of(workspace: &TikiWorkspace, title: &str) -> TaskId {
    workspace
        .island_snapshot()
        .unwrap()
        .open_items
        .iter()
        .find(|item| item.title == title)
        .map(|item| item.task_id)
        .unwrap()
}

/// Every item the island shows anywhere: the Tasks tab, Today's sections, the focus and the
/// Agenda.
fn shown_ids(snapshot: &IslandSnapshot) -> Vec<TaskId> {
    let agenda_items = snapshot
        .agenda
        .iter()
        .flat_map(|day| &day.entries)
        .filter_map(|entry| match entry {
            AgendaEntry::Reminder { item, .. } | AgendaEntry::Task { item, .. } => Some(item),
            AgendaEntry::Meeting { .. } => None,
        });
    snapshot
        .open_items
        .iter()
        .chain(snapshot.sections.iter().flat_map(|section| &section.items))
        .chain(snapshot.focus.iter().map(|focus| &focus.item))
        .chain(agenda_items)
        .map(|item| item.task_id)
        .collect()
}

#[test]
fn a_trashed_item_shows_nowhere_and_never_rings() {
    let workspace = TimedWorkspace::sample_at(TestDay::working_hours());
    let half_past_ten = TestDay::today().and_hms_opt(10, 30, 0).unwrap();
    let call = workspace
        .save_confirmed(reminder_at("Llamar a Lucía", half_past_ten))
        .unwrap();
    let overdue = id_of(&workspace, "Acceso solo a análisis de facturas de Almendra");
    let waiting = id_of(&workspace, "Completar módulo de importaciones");
    workspace.wait_for(waiting, "Lucía").unwrap();
    let trashed = [call, overdue, waiting];
    for task_id in trashed {
        workspace.trash_item(task_id).unwrap();
    }

    let snapshot = workspace.island_snapshot().unwrap();
    assert!(shown_ids(&snapshot).iter().all(|id| !trashed.contains(id)));
    assert_eq!(snapshot.mood, Mood::Hurry);
    assert_eq!(snapshot.open_count, 5);
    let context = workspace.assistant_context().unwrap();
    assert!(
        context
            .open_tasks
            .iter()
            .all(|task| !trashed.contains(&task.task_id))
    );
    let andeanvet = workspace
        .project_summaries()
        .unwrap()
        .into_iter()
        .find(|project| project.name == "Andeanvet")
        .unwrap();
    assert_eq!((andeanvet.open_count, andeanvet.pressing_count), (2, 0));

    let follow_up_day = TestDay::october(24).and_hms_opt(10, 0, 0).unwrap();
    assert_eq!(
        workspace
            .at(follow_up_day)
            .take_due_task_reminders()
            .unwrap(),
        Vec::new()
    );
}

#[test]
fn a_restored_item_comes_back_as_it_was_and_rings_again() {
    let workspace = TimedWorkspace::sample_at(TestDay::working_hours());
    let half_past_ten = TestDay::today().and_hms_opt(10, 30, 0).unwrap();
    let call = workspace
        .save_confirmed(reminder_at("Llamar a Lucía", half_past_ten))
        .unwrap();
    let before = workspace.task_details(call).unwrap();
    workspace.trash_item(call).unwrap();
    assert_eq!(
        workspace
            .at(half_past_ten)
            .take_due_task_reminders()
            .unwrap(),
        Vec::new()
    );

    workspace.restore_item(call).unwrap();

    assert_eq!(workspace.task_details(call).unwrap(), before);
    assert!(shown_ids(&workspace.island_snapshot().unwrap()).contains(&call));
    let alerts = workspace.take_due_task_reminders().unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].task_id, Some(call));
    let events: Vec<ItemEvent> = workspace
        .activity_of(call)
        .unwrap()
        .into_iter()
        .map(|record| record.event)
        .collect();
    assert_eq!(
        events[..3],
        [ItemEvent::Created, ItemEvent::Trashed, ItemEvent::Restored]
    );
}

#[test]
fn an_item_in_the_trash_does_not_change_until_it_is_restored() {
    let workspace = TimedWorkspace::sample_at(TestDay::working_hours());
    let task_id = id_of(&workspace, "Completar módulo de importaciones");
    workspace.trash_item(task_id).unwrap();

    assert!(matches!(
        workspace.move_task(task_id, TaskStatus::Done),
        Err(TikiError::ItemInTrash)
    ));
    assert!(matches!(
        workspace.trash_item(task_id),
        Err(TikiError::ItemInTrash)
    ));
    workspace.restore_item(task_id).unwrap();
    workspace.move_task(task_id, TaskStatus::Done).unwrap();
}

#[test]
fn a_delivered_item_in_the_trash_leaves_the_history_and_the_reports() {
    let workspace = TimedWorkspace::sample_at(TestDay::working_hours());
    let task_id = id_of(&workspace, "Completar módulo de importaciones");
    workspace.deliver(task_id).unwrap();
    assert_eq!(workspace.island_snapshot().unwrap().delivered_today, 1);

    workspace.trash_item(task_id).unwrap();

    assert_eq!(workspace.delivered_tasks().unwrap(), Vec::new());
    assert_eq!(workspace.island_snapshot().unwrap().delivered_today, 0);
    workspace.restore_item(task_id).unwrap();
    assert_eq!(workspace.delivered_tasks().unwrap().len(), 1);
}

#[test]
fn reports_leave_out_the_deliveries_of_items_in_the_trash() {
    let sample = SampleWorkspace::create();
    let database: &Database = &sample.database;
    let noon = TestDay::today().and_hms_opt(12, 0, 0).unwrap();
    let settings = TikiSettings::default();
    let clock = FixedClock::at(noon);
    let items = ItemService::new(database, &settings, &clock);
    let task_id = database
        .tasks()
        .insert(&Task::new("Informe").unwrap(), TestDay::start())
        .unwrap();
    items.move_to(task_id, TaskStatus::Done).unwrap();

    items.trash(task_id).unwrap();

    let activity = database.activity();
    assert_eq!(
        activity
            .deliveries_between(TestDay::today(), TestDay::today(), None)
            .unwrap(),
        Vec::new()
    );
    assert_eq!(activity.count_delivered_on(TestDay::today()).unwrap(), 0);
}
