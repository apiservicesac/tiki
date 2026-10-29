mod support;

use support::{SampleWorkspace, TestDay, TimedWorkspace};
use tiki_core::activity::{ItemEvent, ItemField};
use tiki_core::capture::ConfirmedCapture;
use tiki_core::domain::{ItemKind, Priority, Repeat, Task, TaskStatus};
use tiki_core::editing::TaskEdit;
use tiki_core::storage::Database;

fn capture(title: &str, project_name: Option<&str>) -> ConfirmedCapture {
    ConfirmedCapture {
        title: title.into(),
        project_name: project_name.map(str::to_owned),
        due_date: None,
        priority: Priority::Normal,
        remind_at: None,
        kind: ItemKind::Task,
        tags: Vec::new(),
        repeat: Repeat::Never,
        waiting_for: None,
    }
}

#[test]
fn every_change_to_an_item_is_recorded_in_its_history() {
    let nine = TestDay::today().and_hms_opt(9, 0, 0).unwrap();
    let workspace = TimedWorkspace::sample_at(nine);
    let report = workspace
        .save_confirmed(capture("Informe", Some("Altamira")))
        .unwrap();
    let details = workspace.task_details(report).unwrap();
    workspace
        .edit_task(
            report,
            TaskEdit {
                title: "Informe mensual".into(),
                notes: details.notes,
                project_name: details.project_name,
                due_date: details.due_date,
                priority: Priority::High,
                status: TaskStatus::InProgress,
                remind_at: None,
                kind: details.kind,
                tags: details.tags,
                repeat: details.repeat,
                waiting_for: details.waiting_for,
            },
        )
        .unwrap();
    workspace.move_task(report, TaskStatus::Done).unwrap();
    workspace.move_task(report, TaskStatus::ToDo).unwrap();

    let events: Vec<ItemEvent> = workspace
        .activity_of(report)
        .unwrap()
        .into_iter()
        .map(|record| record.event)
        .collect();
    assert_eq!(
        events,
        [
            ItemEvent::Created,
            ItemEvent::Edited {
                fields: vec![ItemField::Title, ItemField::Priority]
            },
            ItemEvent::Moved {
                from: TaskStatus::ToDo,
                to: TaskStatus::InProgress
            },
            ItemEvent::Delivered,
            ItemEvent::Reopened,
        ]
    );
    assert!(
        workspace
            .activity_of(report)
            .unwrap()
            .iter()
            .all(|record| record.at == nine)
    );
}

#[test]
fn a_reminder_that_rings_closes_without_counting_as_a_delivery() {
    let five = TestDay::today().and_hms_opt(17, 0, 0).unwrap();
    let workspace = TimedWorkspace::sample_at(five);
    let call = workspace
        .save_confirmed(ConfirmedCapture {
            kind: ItemKind::Reminder,
            remind_at: Some(five),
            ..capture("Llamar a mamá", None)
        })
        .unwrap();
    workspace.take_due_task_reminders().unwrap();
    workspace.snooze_for(call, 10).unwrap();

    let events: Vec<ItemEvent> = workspace
        .activity_of(call)
        .unwrap()
        .into_iter()
        .map(|record| record.event)
        .collect();
    assert_eq!(
        events,
        [
            ItemEvent::Created,
            ItemEvent::Rang,
            ItemEvent::Moved {
                from: TaskStatus::ToDo,
                to: TaskStatus::Done
            },
            ItemEvent::Snoozed {
                until: TestDay::today().and_hms_opt(17, 10, 0).unwrap()
            },
            ItemEvent::Moved {
                from: TaskStatus::Done,
                to: TaskStatus::ToDo
            },
        ]
    );
    assert_eq!(workspace.delivered_tasks().unwrap(), Vec::new());
}

#[test]
fn reminders_never_reach_the_delivered_history_or_the_reports() {
    let five = TestDay::today().and_hms_opt(17, 0, 0).unwrap();
    let workspace = TimedWorkspace::sample_at(five);
    let rings = workspace
        .save_confirmed(ConfirmedCapture {
            kind: ItemKind::Reminder,
            remind_at: Some(five),
            ..capture("Llamar a mamá", None)
        })
        .unwrap();
    let checked_off = workspace
        .save_confirmed(ConfirmedCapture {
            kind: ItemKind::Reminder,
            remind_at: Some(TestDay::october(22).and_hms_opt(9, 0, 0).unwrap()),
            ..capture("Regar las plantas", None)
        })
        .unwrap();
    workspace.take_due_task_reminders().unwrap();
    workspace.deliver(checked_off).unwrap();

    assert_eq!(
        workspace.task_details(rings).unwrap().status,
        TaskStatus::Done
    );
    assert_eq!(
        workspace.task_details(checked_off).unwrap().status,
        TaskStatus::Done
    );
    assert_eq!(workspace.delivered_tasks().unwrap(), Vec::new());
    assert_eq!(workspace.island_snapshot().unwrap().delivered_today, 0);
}
