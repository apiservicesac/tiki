mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::TikiError;
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
fn the_edit_form_shows_the_task_with_its_client_name() {
    let workspace = sample_workspace();
    let details = workspace.task_details(first_task_id(&workspace)).unwrap();

    assert_eq!(details.client_name.as_deref(), Some("Andeanvet"));
    assert_eq!(details.status, TaskStatus::ToDo);
}

#[test]
fn editing_a_task_changes_its_fields_and_client() {
    let workspace = sample_workspace();
    let task_id = first_task_id(&workspace);

    workspace
        .edit_task(
            task_id,
            TaskEdit {
                title: "  Acceso a facturas  ".into(),
                notes: "Pedir usuario de prueba".into(),
                client_name: Some("altamira".into()),
                due_date: Some(TestDay::october(30)),
                priority: Priority::Urgent,
                status: TaskStatus::WaitingOnOthers,
            },
        )
        .unwrap();

    let details = workspace.task_details(task_id).unwrap();
    assert_eq!(details.title, "Acceso a facturas");
    assert_eq!(details.notes, "Pedir usuario de prueba");
    assert_eq!(details.client_name.as_deref(), Some("Altamira"));
    assert_eq!(details.due_date, Some(TestDay::october(30)));
    assert_eq!(details.priority, Priority::Urgent);
    assert_eq!(details.status, TaskStatus::WaitingOnOthers);
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
            client_name: None,
            due_date: None,
            priority: details.priority,
            status: details.status,
        },
    );
    assert!(matches!(result, Err(TikiError::EmptyTaskTitle)));
}
