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
fn snoozing_a_reminder_that_rang_keeps_its_delivery_in_the_history() {
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
    workspace.snooze(call, 10).unwrap();

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
            ItemEvent::Delivered,
            ItemEvent::Snoozed {
                until: TestDay::today().and_hms_opt(17, 10, 0).unwrap()
            },
            ItemEvent::Reopened,
        ]
    );
    assert_eq!(workspace.delivered_tasks().unwrap()[0].task_id, call);
}

#[test]
fn reports_read_the_deliveries_between_two_days_by_project() {
    let sample = SampleWorkspace::create();
    let database: &Database = &sample.database;
    let deliver = |title: &str, project, day| {
        let mut task = Task::new(title).unwrap();
        task.project_id = project;
        let task_id = database.tasks().insert(&task).unwrap();
        database
            .activity()
            .record(
                task_id,
                TestDay::october(day).and_hms_opt(12, 0, 0).unwrap(),
                &[ItemEvent::Delivered],
            )
            .unwrap();
        task_id
    };
    let early = deliver("Antes", Some(sample.altamira), 19);
    let altamira = deliver("Altamira", Some(sample.altamira), 20);
    let andeanvet = deliver("Andeanvet", Some(sample.andeanvet), 21);

    let activity = database.activity();
    let week: Vec<_> = activity
        .deliveries_between(TestDay::october(20), TestDay::october(21), None)
        .unwrap()
        .into_iter()
        .map(|delivery| delivery.item_id)
        .collect();
    assert_eq!(week, [altamira, andeanvet]);
    let for_altamira: Vec<_> = activity
        .deliveries_between(
            TestDay::october(19),
            TestDay::october(21),
            Some(sample.altamira),
        )
        .unwrap()
        .into_iter()
        .map(|delivery| delivery.item_id)
        .collect();
    assert_eq!(for_altamira, [early, altamira]);
    assert_eq!(
        activity.count_delivered_on(TestDay::october(21)).unwrap(),
        1
    );
}
