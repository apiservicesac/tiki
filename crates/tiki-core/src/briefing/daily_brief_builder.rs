use chrono::NaiveDate;

use super::{BriefDetail, BriefItem, DailyBrief};
use crate::domain::{ClientDirectory, Task, TaskId, TaskStatus};
use crate::settings::TikiSettings;

/// Sorts open tasks into the sections of the daily brief.
pub struct DailyBriefBuilder<'settings> {
    settings: &'settings TikiSettings,
}

impl<'settings> DailyBriefBuilder<'settings> {
    pub fn new(settings: &'settings TikiSettings) -> Self {
        Self { settings }
    }

    pub fn build(&self, tasks: &[Task], clients: &ClientDirectory, today: NaiveDate) -> DailyBrief {
        let mut brief = DailyBrief::empty(today);
        for task in tasks.iter().filter(|task| task.status.is_open()) {
            if let Some(task_id) = task.id {
                self.place(&mut brief, task, task_id, clients);
            }
        }
        brief
    }

    /// Puts a task in exactly one section, checked in this order:
    /// waiting on others, overdue, due soon, and finally "your turn".
    fn place(
        &self,
        brief: &mut DailyBrief,
        task: &Task,
        task_id: TaskId,
        clients: &ClientDirectory,
    ) {
        let today = brief.date;
        let line = |detail| Self::line(task, task_id, clients, detail);

        if let Some(days) = task.days_waiting(today) {
            let waiting_line = line(BriefDetail::Waiting { days });
            if days >= self.settings.follow_up_after_days {
                brief.needs_follow_up.push(waiting_line.clone());
            }
            brief.waiting.push(waiting_line);
        } else if task.is_overdue(today) {
            let days_late = -task.days_until_due(today).unwrap_or_default();
            brief
                .overdue
                .push(line(BriefDetail::Overdue { days: days_late }));
        } else if task.is_due_within(today, self.settings.due_soon_window_days) {
            let days = task.days_until_due(today).unwrap_or_default();
            brief.due_soon.push(line(BriefDetail::DueIn { days }));
        } else if task.status.is_on_user() {
            brief
                .your_turn
                .push(line(Self::status_or_due_detail(task, today)));
        }
    }

    fn line(
        task: &Task,
        task_id: TaskId,
        clients: &ClientDirectory,
        detail: BriefDetail,
    ) -> BriefItem {
        BriefItem {
            task_id,
            title: task.title.clone(),
            client_name: clients.name_of(task.client_id).map(str::to_owned),
            detail,
        }
    }

    fn status_or_due_detail(task: &Task, today: NaiveDate) -> BriefDetail {
        match task.days_until_due(today) {
            Some(days) if task.status == TaskStatus::ToDo => BriefDetail::DueIn { days },
            _ => BriefDetail::InStatus {
                status: task.status,
            },
        }
    }
}
