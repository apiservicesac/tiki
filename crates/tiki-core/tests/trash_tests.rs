mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::activity::ItemEvent;
use tiki_core::domain::Task;

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
