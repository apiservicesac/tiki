mod support;

use support::TestDay;
use tiki_core::TikiError;
use tiki_core::domain::{ItemKind, Repeat, Task, TaskStatus};

#[test]
fn a_task_needs_a_title() {
    assert!(matches!(Task::new("   "), Err(TikiError::EmptyTaskTitle)));
}

#[test]
fn moving_to_waiting_remembers_the_day_and_leaving_it_forgets_it() {
    let mut task = Task::new("Permisos de GitHub").unwrap();

    task.move_to(TaskStatus::Waiting, TestDay::october(18));
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

fn reminder_at(title: &str, at: chrono::NaiveDateTime, repeat: Repeat) -> Task {
    let mut reminder = Task::new(title).unwrap();
    reminder.kind = ItemKind::Reminder;
    reminder.remind_at = Some(at);
    reminder.repeat = repeat;
    reminder
}

#[test]
fn a_one_time_reminder_is_done_once_it_rings() {
    let five = TestDay::today().and_hms_opt(17, 0, 0).unwrap();
    let mut call = reminder_at("Llamar a mamá", five, Repeat::Never);

    call.ring(five);

    assert_eq!(call.status, TaskStatus::Done);
    assert_eq!(call.remind_at, None);
}

#[test]
fn a_repeating_reminder_moves_to_its_next_occurrence_when_it_rings() {
    let eight = TestDay::today().and_hms_opt(8, 0, 0).unwrap();
    let mut pill = reminder_at("Pastilla", eight, Repeat::Daily);

    pill.ring(eight);

    assert_eq!(pill.status, TaskStatus::ToDo);
    assert_eq!(
        pill.remind_at,
        Some(TestDay::october(22).and_hms_opt(8, 0, 0).unwrap())
    );
}

#[test]
fn the_alarm_on_a_task_rings_once_and_leaves_the_task_open() {
    let ten = TestDay::today().and_hms_opt(10, 0, 0).unwrap();
    let mut task = Task::new("Revisar acta").unwrap();
    task.remind_at = Some(ten);

    task.ring(ten);

    assert_eq!(task.status, TaskStatus::ToDo);
    assert_eq!(task.remind_at, None);
}

#[test]
fn snoozing_a_reminder_that_rang_brings_it_back_at_the_new_time() {
    let five = TestDay::today().and_hms_opt(17, 0, 0).unwrap();
    let later = TestDay::today().and_hms_opt(17, 10, 0).unwrap();
    let mut call = reminder_at("Llamar a mamá", five, Repeat::Never);
    call.ring(five);

    call.snooze(later);

    assert_eq!(call.status, TaskStatus::ToDo);
    assert_eq!(call.remind_at, Some(later));
}
