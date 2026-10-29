mod support;

use chrono::NaiveDateTime;
use support::{SampleWorkspace, TestDay};
use tiki_core::capture::ConfirmedCapture;
use tiki_core::domain::{ItemKind, Priority, Repeat, Task, TaskId, TaskStatus};
use tiki_core::editing::TaskEdit;
use tiki_core::workspace::{FixedClock, TikiWorkspace};
use tiki_core::{TikiError, TikiSettings};

fn sample_workspace() -> TikiWorkspace {
    TikiWorkspace::new(
        SampleWorkspace::create().database,
        TikiSettings::default(),
        Box::new(FixedClock::new(TestDay::today())),
    )
    .unwrap()
}

fn capture(title: &str, kind: ItemKind, remind_at: Option<NaiveDateTime>) -> ConfirmedCapture {
    ConfirmedCapture {
        title: title.into(),
        project_name: None,
        due_date: None,
        priority: Priority::Normal,
        remind_at,
        kind,
        tags: Vec::new(),
        repeat: Repeat::Daily,
        waiting_for: None,
    }
}

fn edit_of(workspace: &TikiWorkspace, task_id: TaskId) -> TaskEdit {
    let details = workspace.task_details(task_id).unwrap();
    TaskEdit {
        title: details.title,
        notes: details.notes,
        project_name: details.project_name,
        due_date: details.due_date,
        priority: details.priority,
        status: details.status,
        remind_at: details.remind_at,
        kind: details.kind,
        tags: details.tags,
        repeat: details.repeat,
        waiting_for: details.waiting_for,
    }
}

fn five_pm() -> NaiveDateTime {
    TestDay::today().and_hms_opt(17, 0, 0).unwrap()
}

#[test]
fn a_reminder_cannot_be_captured_without_a_time() {
    let workspace = sample_workspace();
    let saved = workspace.save_confirmed(capture("Llamar", ItemKind::Reminder, None));
    assert!(matches!(saved, Err(TikiError::ReminderNeedsTime)));
}

#[test]
fn a_reminder_cannot_lose_its_time_in_the_edit_form() {
    let workspace = sample_workspace();
    let call = workspace
        .save_confirmed(capture("Llamar", ItemKind::Reminder, Some(five_pm())))
        .unwrap();

    let result = workspace.edit_task(
        call,
        TaskEdit {
            remind_at: None,
            ..edit_of(&workspace, call)
        },
    );
    assert!(matches!(result, Err(TikiError::ReminderNeedsTime)));
}

#[test]
fn only_a_reminder_with_a_time_repeats() {
    let workspace = sample_workspace();
    let task = workspace
        .save_confirmed(capture("Informe", ItemKind::Task, Some(five_pm())))
        .unwrap();
    let reminder = workspace
        .save_confirmed(capture("Agua", ItemKind::Reminder, Some(five_pm())))
        .unwrap();

    assert_eq!(workspace.task_details(task).unwrap().repeat, Repeat::Never);
    assert_eq!(
        workspace.task_details(reminder).unwrap().repeat,
        Repeat::Daily
    );
}

#[test]
fn leaving_waiting_forgets_who_it_waited_for_and_the_automatic_nudge() {
    let workspace = sample_workspace();
    let contract = workspace
        .save_confirmed(ConfirmedCapture {
            waiting_for: Some("Lucía".into()),
            repeat: Repeat::Never,
            ..capture("Contrato", ItemKind::Task, None)
        })
        .unwrap();
    assert!(
        workspace
            .task_details(contract)
            .unwrap()
            .follow_up_at
            .is_some()
    );

    workspace.move_task(contract, TaskStatus::ToDo).unwrap();

    let details = workspace.task_details(contract).unwrap();
    assert_eq!(details.waiting_for, "");
    assert_eq!(details.follow_up_at, None);
}

#[test]
fn leaving_waiting_keeps_a_reminder_the_user_set() {
    let workspace = sample_workspace();
    let contract = workspace
        .save_confirmed(ConfirmedCapture {
            waiting_for: Some("Lucía".into()),
            repeat: Repeat::Never,
            ..capture("Contrato", ItemKind::Task, Some(five_pm()))
        })
        .unwrap();

    workspace
        .move_task(contract, TaskStatus::InProgress)
        .unwrap();

    assert_eq!(
        workspace.task_details(contract).unwrap().remind_at,
        Some(five_pm())
    );
}

#[test]
fn normalizing_trims_the_text_and_clears_what_does_not_apply() {
    let mut task = Task::new("Informe").unwrap();
    task.title = "  Informe mensual ".into();
    task.tags = vec!["#Cierre".into()];
    task.waiting_for = "Lucía".into();
    task.repeat = Repeat::Weekly;

    task.normalize().unwrap();

    assert_eq!(task.title, "Informe mensual");
    assert_eq!(task.tags, ["cierre"]);
    assert_eq!(task.waiting_for, "");
    assert_eq!(task.repeat, Repeat::Never);
}
