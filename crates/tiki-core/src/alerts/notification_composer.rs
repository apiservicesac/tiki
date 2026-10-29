use chrono::NaiveDate;

use super::{AlertKind, TikiAlert};
use crate::briefing::{BriefDetail, BriefDetailFormatter, BriefItem, BriefMoment, DailyBrief};
use crate::calendar::MeetingItem;
use crate::domain::{ItemKind, Task};
use crate::i18n::{DateFormatter, Translator};

/// Writes the short notification for a brief: counts first, then what to start with and
/// who to follow up.
pub struct NotificationComposer<'translator> {
    translator: &'translator Translator,
}

impl<'translator> NotificationComposer<'translator> {
    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    /// The reminder the user set on an item: a standalone reminder, or an alarm on a task.
    pub fn compose_reminder(
        &self,
        task: &Task,
        project_name: Option<&str>,
        today: NaiveDate,
    ) -> TikiAlert {
        let kind = match task.kind {
            ItemKind::Reminder => AlertKind::Reminder,
            ItemKind::Task => AlertKind::Task,
        };
        self.compose_item(kind, task, project_name, today)
    }

    /// The nudge to follow up an item that still waits on others.
    pub fn compose_follow_up(
        &self,
        task: &Task,
        project_name: Option<&str>,
        today: NaiveDate,
    ) -> TikiAlert {
        self.compose_item(AlertKind::FollowUp, task, project_name, today)
    }

    /// One item's alert: its title, its project, who it waits for and when it is due.
    fn compose_item(
        &self,
        kind: AlertKind,
        task: &Task,
        project_name: Option<&str>,
        today: NaiveDate,
    ) -> TikiAlert {
        let mut lines = vec![task.title.clone()];
        let mut context: Vec<String> = project_name.map(str::to_owned).into_iter().collect();
        if kind == AlertKind::FollowUp {
            let days = task.days_waiting(today).unwrap_or_default().to_string();
            context.push(if task.waiting_for.is_empty() {
                self.translator
                    .format("notify.follow_up.days", &[("days", &days)])
            } else {
                self.translator.format(
                    "notify.follow_up.waiting_for",
                    &[("who", &task.waiting_for), ("days", &days)],
                )
            });
        }
        if let Some(days) = task.days_until_due(today) {
            let detail = if days < 0 {
                BriefDetail::Overdue { days: -days }
            } else {
                BriefDetail::DueIn { days }
            };
            context.push(BriefDetailFormatter::new(self.translator).label(detail, today));
        }
        if !context.is_empty() {
            lines.push(context.join(" · "));
        }
        TikiAlert {
            kind,
            title: self
                .translator
                .text(&format!("notify.{}.title", Self::alert_key(kind))),
            body: lines.join("\n"),
            task_id: task.id,
            link: None,
        }
    }

    /// "Meeting in 12 min": its title, its time and where.
    pub fn compose_meeting(&self, meeting: &MeetingItem, now: chrono::NaiveDateTime) -> TikiAlert {
        let minutes = (meeting.starts_at - now).num_minutes().max(0);
        let mut details = vec![
            DateFormatter::new(self.translator)
                .time_range(meeting.starts_at.time(), meeting.ends_at.time()),
        ];
        if !meeting.location.is_empty() {
            details.push(meeting.location.clone());
        }
        TikiAlert {
            kind: AlertKind::Meeting,
            title: self
                .translator
                .format("notify.meeting.title", &[("minutes", &minutes.to_string())]),
            body: format!("{}\n{}", meeting.title, details.join(" · ")),
            task_id: None,
            link: meeting.join_url.clone(),
        }
    }

    /// "Meetings today: 3 · the first at 10:00", for the morning brief.
    pub fn meetings_line(&self, meetings_today: &[&MeetingItem]) -> Option<String> {
        let first = meetings_today.iter().find(|meeting| !meeting.all_day)?;
        Some(self.translator.format(
            "notify.meetings_today",
            &[
                ("count", &meetings_today.len().to_string()),
                ("time", &DateFormatter::time(first.starts_at.time())),
            ],
        ))
    }

    fn alert_key(kind: AlertKind) -> &'static str {
        match kind {
            AlertKind::Reminder => "reminder",
            AlertKind::FollowUp => "follow_up",
            _ => "task",
        }
    }

    pub fn compose(&self, brief: &DailyBrief, moment: BriefMoment) -> TikiAlert {
        let lines = match moment {
            BriefMoment::Morning => self.morning_lines(brief),
            BriefMoment::Evening => self.evening_lines(brief),
        };
        TikiAlert {
            kind: AlertKind::from(moment),
            task_id: None,
            link: None,
            title: self
                .translator
                .text(&format!("notify.{}.title", moment.key())),
            body: lines.join("\n"),
        }
    }

    fn morning_lines(&self, brief: &DailyBrief) -> Vec<String> {
        let mut lines = Vec::new();
        if brief.is_all_clear() {
            lines.push(self.translator.text("notify.all_clear"));
        } else {
            lines.push(self.counts(brief));
        }
        if let Some(first) = brief.overdue.first().or(brief.due_soon.first()) {
            lines.push(
                self.translator
                    .format("notify.start_with", &[("item", &self.item(first))]),
            );
        }
        lines.extend(self.list_line("notify.follow_up", &brief.needs_follow_up));
        lines
    }

    fn evening_lines(&self, brief: &DailyBrief) -> Vec<String> {
        let mut lines = vec![self.counts(brief)];
        lines.extend(self.list_line("notify.tomorrow", &brief.due_tomorrow()));
        lines.extend(self.list_line("notify.follow_up", &brief.needs_follow_up));
        lines
    }

    fn counts(&self, brief: &DailyBrief) -> String {
        self.translator.format(
            "notify.counts",
            &[
                ("overdue", &brief.overdue.len().to_string()),
                ("due", &brief.due_soon.len().to_string()),
                ("waiting", &brief.waiting.len().to_string()),
            ],
        )
    }

    fn list_line(&self, key: &str, items: &[BriefItem]) -> Option<String> {
        if items.is_empty() {
            return None;
        }
        let names: Vec<String> = items.iter().map(|item| self.item(item)).collect();
        Some(
            self.translator
                .format(key, &[("items", &self.translator.join_list(&names))]),
        )
    }

    fn item(&self, item: &BriefItem) -> String {
        match &item.project_name {
            Some(project) => self.translator.format(
                "narration.item_with_project",
                &[("title", &item.title), ("project", project)],
            ),
            None => item.title.clone(),
        }
    }
}
