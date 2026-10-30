use chrono::NaiveDateTime;

use crate::activity::{ItemEvent, ItemField};
use crate::capture::ConfirmedCapture;
use crate::domain::{FollowUpPolicy, Task, TaskId, TaskStatus};
use crate::editing::TaskEdit;
use crate::error::{TikiError, TikiResult};
use crate::projects::ProjectBook;
use crate::settings::TikiSettings;
use crate::storage::{Database, Repositories};
use crate::workspace::Clock;

/// Applies the user's changes to items. Every change follows the same path: the item's own
/// rules (`Task::normalize`), its follow-up nudge, and one transaction that saves the item
/// and the events of the change.
pub struct ItemService<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
    clock: &'context dyn Clock,
}

impl<'context> ItemService<'context> {
    pub fn new(
        database: &'context Database,
        settings: &'context TikiSettings,
        clock: &'context dyn Clock,
    ) -> Self {
        Self {
            database,
            settings,
            clock,
        }
    }

    /// Saves an item exactly as the user confirmed it; a project named for the first time is
    /// created on the spot.
    pub fn capture(&self, capture: ConfirmedCapture) -> TikiResult<TaskId> {
        let project_id = ProjectBook::new(self.database, self.settings)
            .find_or_create(capture.project_name.as_deref())?;
        let task = capture.into_task(project_id, self.clock.today())?;
        self.create(task, |_, _| Ok(()))
    }

    /// Saves a new item with its first nudge and its creation event; `alongside` saves what
    /// points to the new item (such as the session proposal it came from) in the same
    /// transaction.
    pub fn create(
        &self,
        mut task: Task,
        alongside: impl FnOnce(&Repositories<'_>, TaskId) -> TikiResult<()>,
    ) -> TikiResult<TaskId> {
        FollowUpPolicy::new(self.settings).schedule_first_nudge(&mut task, false);
        task.normalize()?;
        let now = self.clock.now();
        self.database.transaction(|repositories| {
            let task_id = repositories.tasks().insert(&task, now)?;
            repositories
                .activity()
                .record(task_id, now, &[ItemEvent::Created])?;
            alongside(repositories, task_id)?;
            Ok(task_id)
        })
    }

    /// Saves the user's changes to an item; a new project name creates the project.
    pub fn edit(&self, task_id: TaskId, edit: TaskEdit) -> TikiResult<()> {
        let project_id = ProjectBook::new(self.database, self.settings)
            .find_or_create(edit.project_name.as_deref())?;
        self.change(task_id, |task, now| {
            let before = task.clone();
            let moved = edit.apply_to(task, project_id, now.date());
            // Normalized first, so the history lists only the fields that really changed.
            task.normalize()?;
            let fields = ItemField::changed_between(&before, task);
            let edited = (!fields.is_empty()).then_some(ItemEvent::Edited { fields });
            Ok(edited.into_iter().chain(moved).collect())
        })
    }

    pub fn move_to(&self, task_id: TaskId, status: TaskStatus) -> TikiResult<()> {
        self.change(task_id, |task, now| {
            Ok(task.move_to(status, now.date()).into_iter().collect())
        })
    }

    /// Brings a delivered item back to "to do"; an open item stays as it is.
    pub fn reopen(&self, task_id: TaskId) -> TikiResult<()> {
        self.change(task_id, |task, now| {
            Ok(if task.status == TaskStatus::Done {
                task.move_to(TaskStatus::ToDo, now.date())
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            })
        })
    }

    /// Puts the item on hold until `who` answers (free text, as the user says it).
    pub fn wait_for(&self, task_id: TaskId, who: &str) -> TikiResult<()> {
        self.change(task_id, |task, now| {
            let who = who.trim();
            let changed_who = task.waiting_for != who;
            who.clone_into(&mut task.waiting_for);
            let moved = task.move_to(TaskStatus::Waiting, now.date());
            let edited = (moved.is_none() && changed_who).then(|| ItemEvent::Edited {
                fields: vec![ItemField::WaitingFor],
            });
            Ok(moved.into_iter().chain(edited).collect())
        })
    }

    /// Puts the item's reminder off until `until`; one that already rang comes back.
    pub fn snooze(&self, task_id: TaskId, until: NaiveDateTime) -> TikiResult<()> {
        self.change(task_id, |task, _| Ok(task.snooze(until)))
    }

    /// Sends the item to the trash: it shows nowhere else and its reminders stay silent until
    /// it is restored.
    pub fn trash(&self, task_id: TaskId) -> TikiResult<()> {
        self.change(task_id, |task, now| {
            Ok(task.trash(now).into_iter().collect())
        })
    }

    /// Takes the item out of the trash exactly as it was; its reminders ring again.
    pub fn restore(&self, task_id: TaskId) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        let events: Vec<ItemEvent> = task.restore().into_iter().collect();
        self.save(&task, self.clock.now(), &events)
    }

    /// Loads an item outside the trash, applies `apply` (which returns the events of the
    /// change), keeps the item's rules and saves it with its events. An item in the trash
    /// does not change until it is restored.
    fn change(
        &self,
        task_id: TaskId,
        apply: impl FnOnce(&mut Task, NaiveDateTime) -> TikiResult<Vec<ItemEvent>>,
    ) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        if task.is_trashed() {
            return Err(TikiError::ItemInTrash);
        }
        let was_waiting = task.status == TaskStatus::Waiting;
        let now = self.clock.now();
        let events = apply(&mut task, now)?;
        FollowUpPolicy::new(self.settings).schedule_first_nudge(&mut task, was_waiting);
        task.normalize()?;
        self.save(&task, now, &events)
    }

    /// Saves the item with the events of its change in one transaction.
    fn save(&self, task: &Task, now: NaiveDateTime, events: &[ItemEvent]) -> TikiResult<()> {
        let task_id = task.id.ok_or(TikiError::UnsavedTask)?;
        self.database.transaction(|repositories| {
            repositories.tasks().update(task, now)?;
            repositories.activity().record(task_id, now, events)
        })
    }
}
