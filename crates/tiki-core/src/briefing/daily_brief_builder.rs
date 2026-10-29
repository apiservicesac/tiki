use chrono::NaiveDate;

use super::{BriefDetail, BriefItem, DailyBrief};
use crate::domain::{FollowUpPolicy, ItemKind, ProjectDirectory, Task, TaskId, TaskStatus};
use crate::settings::TikiSettings;

/// Sorts open tasks into the sections of the daily brief.
pub struct DailyBriefBuilder<'settings> {
    settings: &'settings TikiSettings,
}

impl<'settings> DailyBriefBuilder<'settings> {
    pub fn new(settings: &'settings TikiSettings) -> Self {
        Self { settings }
    }

    pub fn build(
        &self,
        tasks: &[Task],
        projects: &ProjectDirectory,
        today: NaiveDate,
    ) -> DailyBrief {
        let mut brief = DailyBrief::empty(today);
        for task in tasks.iter().filter(|task| task.status.is_open()) {
            if let Some(task_id) = task.id {
                self.place(&mut brief, task, task_id, projects);
            }
        }
        // Within each section, the most important first; the stable sort keeps the date order.
        for section in [
            &mut brief.overdue,
            &mut brief.due_soon,
            &mut brief.your_turn,
            &mut brief.waiting,
            &mut brief.needs_follow_up,
        ] {
            section.sort_by_key(|item| std::cmp::Reverse(item.priority));
        }
        brief.reminders.sort_by_key(BriefItem::rings_at);
        brief
    }

    /// Puts a task in exactly one section, checked in this order:
    /// waiting on others, overdue, due soon, and finally "your turn".
    fn place(
        &self,
        brief: &mut DailyBrief,
        task: &Task,
        task_id: TaskId,
        projects: &ProjectDirectory,
    ) {
        let today = brief.date;
        let line = |detail| Self::line(task, task_id, projects, detail);

        // Standalone reminders are not work: they only list when they ring.
        if task.kind == ItemKind::Reminder {
            if let Some(at) = task.remind_at {
                brief.reminders.push(line(BriefDetail::RemindAt {
                    at,
                    repeat: task.repeat,
                }));
            }
            return;
        }

        if let Some(days) = task.days_waiting(today) {
            let waiting_line = line(BriefDetail::Waiting { days });
            if FollowUpPolicy::new(self.settings).is_due(days) {
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
        projects: &ProjectDirectory,
        detail: BriefDetail,
    ) -> BriefItem {
        BriefItem {
            task_id,
            title: task.title.clone(),
            project_name: projects.name_of(task.project_id).map(str::to_owned),
            tags: task.tags.clone(),
            priority: task.priority,
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
