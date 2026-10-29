mod support;

use tiki_core::domain::ItemKind;

use support::{SampleWorkspace, TestDay};
use tiki_core::TikiError;
use tiki_core::TikiSettings;
use tiki_core::domain::{Priority, TaskId, TaskStatus};
use tiki_core::editing::TaskEdit;
use tiki_core::workspace::{FixedClock, TikiWorkspace};

fn sample_workspace() -> TikiWorkspace {
    TikiWorkspace::new(
        SampleWorkspace::create().database,
        TikiSettings::default(),
        Box::new(FixedClock::new(TestDay::today())),
    )
    .unwrap()
}

fn first_task_id(workspace: &TikiWorkspace) -> TaskId {
    workspace.island_snapshot().unwrap().sections[0].items[0].task_id
}

#[test]
fn the_edit_form_shows_the_task_with_its_project_name() {
    let workspace = sample_workspace();
    let details = workspace.task_details(first_task_id(&workspace)).unwrap();

    assert_eq!(details.project_name.as_deref(), Some("Andeanvet"));
    assert_eq!(details.status, TaskStatus::ToDo);
}

#[test]
fn editing_a_task_changes_its_fields_and_project() {
    let workspace = sample_workspace();
    let task_id = first_task_id(&workspace);

    workspace
        .edit_task(
            task_id,
            TaskEdit {
                title: "  Acceso a facturas  ".into(),
                notes: "Pedir usuario de prueba".into(),
                project_name: Some("altamira".into()),
                due_date: Some(TestDay::october(30)),
                priority: Priority::Urgent,
                status: TaskStatus::Waiting,
                remind_at: Some(TestDay::october(29).and_hms_opt(16, 0, 0).unwrap()),
                kind: ItemKind::Task,
                tags: Vec::new(),
                repeat: tiki_core::domain::Repeat::Never,
                waiting_for: String::new(),
            },
        )
        .unwrap();

    let details = workspace.task_details(task_id).unwrap();
    assert_eq!(details.title, "Acceso a facturas");
    assert_eq!(details.notes, "Pedir usuario de prueba");
    assert_eq!(details.project_name.as_deref(), Some("Altamira"));
    assert_eq!(details.due_date, Some(TestDay::october(30)));
    assert_eq!(details.priority, Priority::Urgent);
    assert_eq!(details.status, TaskStatus::Waiting);
    assert_eq!(
        details.remind_at,
        Some(TestDay::october(29).and_hms_opt(16, 0, 0).unwrap())
    );
}

#[test]
fn a_task_cannot_lose_its_title() {
    let workspace = sample_workspace();
    let task_id = first_task_id(&workspace);
    let details = workspace.task_details(task_id).unwrap();

    let result = workspace.edit_task(
        task_id,
        TaskEdit {
            title: "   ".into(),
            notes: details.notes,
            project_name: None,
            due_date: None,
            priority: details.priority,
            status: details.status,
            remind_at: None,
            kind: ItemKind::Task,
            tags: Vec::new(),
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: String::new(),
        },
    );
    assert!(matches!(result, Err(TikiError::EmptyTaskTitle)));
}

#[test]
fn a_delivery_stays_in_the_history_after_reopening() {
    let workspace = sample_workspace();
    let task_id = first_task_id(&workspace);
    workspace.move_task(task_id, TaskStatus::Done).unwrap();

    let history = workspace.delivered_tasks().unwrap();
    assert_eq!(history[0].task_id, task_id);
    assert_eq!(history[0].delivered_label, "hoy");
    assert_eq!(history[0].project_name.as_deref(), Some("Andeanvet"));
    assert_eq!(workspace.island_snapshot().unwrap().delivered_today, 1);

    workspace
        .move_task(task_id, TaskStatus::InProgress)
        .unwrap();
    assert_eq!(workspace.delivered_tasks().unwrap()[0].task_id, task_id);
    assert_eq!(workspace.island_snapshot().unwrap().delivered_today, 1);
    let details = workspace.task_details(task_id).unwrap();
    assert_eq!(details.status, TaskStatus::InProgress);
}

#[test]
fn a_delivered_item_keeps_its_project_and_tags() {
    let workspace = sample_workspace();
    let task_id = first_task_id(&workspace);
    let details = workspace.task_details(task_id).unwrap();
    workspace
        .edit_task(
            task_id,
            TaskEdit {
                title: details.title,
                notes: details.notes,
                project_name: details.project_name,
                due_date: details.due_date,
                priority: details.priority,
                status: details.status,
                remind_at: details.remind_at,
                kind: details.kind,
                tags: vec!["facturas".into()],
                repeat: details.repeat,
                waiting_for: details.waiting_for,
            },
        )
        .unwrap();
    workspace.deliver(task_id).unwrap();

    let delivered = &workspace.delivered_tasks().unwrap()[0];
    assert_eq!(delivered.tags, ["facturas"]);
    assert!(delivered.project_id.is_some());
}
