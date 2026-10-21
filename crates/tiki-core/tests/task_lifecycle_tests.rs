mod support;

use support::TestDay;
use tiki_core::TikiError;
use tiki_core::domain::{Task, TaskStatus};

#[test]
fn a_task_needs_a_title() {
    assert!(matches!(Task::new("   "), Err(TikiError::EmptyTaskTitle)));
}

#[test]
fn moving_to_waiting_remembers_the_day_and_leaving_it_forgets_it() {
    let mut task = Task::new("Permisos de GitHub").unwrap();

    task.move_to(TaskStatus::WaitingOnOthers, TestDay::october(18));
    assert_eq!(task.waiting_since, Some(TestDay::october(18)));
    assert_eq!(task.days_waiting(TestDay::today()), Some(3));

    task.move_to(TaskStatus::InProgress, TestDay::today());
    assert_eq!(task.waiting_since, None);
    assert_eq!(task.days_waiting(TestDay::today()), None);
}

#[test]
fn overdue_and_due_soon_depend_on_today() {
    let late = Task::new("Late")
        .unwrap()
        .with_due_date(TestDay::october(20));
    let tomorrow = Task::new("Tomorrow")
        .unwrap()
        .with_due_date(TestDay::october(22));

    assert!(late.is_overdue(TestDay::today()));
    assert!(!tomorrow.is_overdue(TestDay::today()));
    assert!(tomorrow.is_due_within(TestDay::today(), 1));
    assert!(!tomorrow.is_due_within(TestDay::today(), 0));
}

#[test]
fn delivered_tasks_are_never_overdue() {
    let mut delivered = Task::new("Delivered")
        .unwrap()
        .with_due_date(TestDay::october(1));
    delivered.move_to(TaskStatus::Done, TestDay::today());
    assert!(!delivered.is_overdue(TestDay::today()));
}
