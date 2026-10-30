use chrono::NaiveDate;

use super::{AlertAction, AlertKind, MeetingOrigin, TikiAlert};
use crate::briefing::{BriefDetail, BriefDetailFormatter, BriefItem, BriefMoment, DailyBrief};
use crate::calendar::MeetingItem;
use crate::domain::{ItemKind, Task};
use crate::i18n::{DateFormatter, Translator};
use crate::sessions::RecordingCue;

/// Writes every alert in the user's language: the morning brief and the end-of-day close,
/// item reminders, follow-up nudges and meetings starting soon.
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
        TikiAlert {
            kind,
            title: self
                .translator
                .text(&format!("notify.{}.title", Self::alert_key(kind))),
            summary: task.title.clone(),
            tips: (!context.is_empty())
                .then(|| context.join(" · "))
                .into_iter()
                .collect(),
            actions: AlertAction::offered_for(kind, false),
            task_id: task.id,
            link: None,
            origin: None,
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
            summary: meeting.title.clone(),
            tips: vec![details.join(" · ")],
            actions: AlertAction::offered_for(AlertKind::Meeting, meeting.join_url.is_some()),
            task_id: None,
            link: meeting.join_url.clone(),
            origin: Some(self.meeting_origin(meeting)),
        }
    }

    /// The meeting's account and platform, and both in one line.
    fn meeting_origin(&self, meeting: &MeetingItem) -> MeetingOrigin {
        let platform = meeting.platform.map(|platform| {
            self.translator
                .text(&format!("meeting.platform.{}", platform.key()))
        });
        let name = Some(meeting.account.display_name.clone()).filter(|name| !name.is_empty());
        let label = [platform, name, Some(meeting.account.email.clone())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        MeetingOrigin {
            account: meeting.account.clone(),
            platform: meeting.platform,
            label,
        }
    }

    /// "Record this call?": what it is about, and the reminder to tell the participants.
    pub fn compose_recording(&self, cue: &RecordingCue) -> TikiAlert {
        let (title, summary) = match cue {
            RecordingCue::Call => (
                self.translator.text("notify.recording.call.title"),
                self.translator.text("notify.recording.call.summary"),
            ),
            RecordingCue::Meeting { title } => (
                self.translator.text("notify.recording.meeting.title"),
                title.clone(),
            ),
        };
        TikiAlert {
            kind: AlertKind::Recording,
            title,
            summary,
            tips: vec![self.translator.text("notify.recording.consent")],
            actions: AlertAction::offered_for(AlertKind::Recording, false),
            task_id: None,
            link: None,
            origin: None,
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

    /// The morning brief or the end-of-day close: the counts, then what to start with,
    /// what is due tomorrow and whom to follow up.
    pub fn compose(&self, brief: &DailyBrief, moment: BriefMoment) -> TikiAlert {
        let (summary, tips) = match moment {
            BriefMoment::Morning => self.morning_lines(brief),
            BriefMoment::Evening => self.evening_lines(brief),
        };
        let kind = AlertKind::from(moment);
        TikiAlert {
            kind,
            title: self
                .translator
                .text(&format!("notify.{}.title", moment.key())),
            summary,
            tips,
            actions: AlertAction::offered_for(kind, false),
            task_id: None,
            link: None,
            origin: None,
        }
    }

    /// The counts, or "all clear", and the tips of the morning.
    fn morning_lines(&self, brief: &DailyBrief) -> (String, Vec<String>) {
        let summary = if brief.is_all_clear() {
            self.translator.text("notify.all_clear")
        } else {
            self.counts(brief)
        };
        let mut tips = Vec::new();
        if let Some(first) = brief.overdue.first().or(brief.due_soon.first()) {
            tips.push(
                self.translator
                    .format("notify.start_with", &[("item", &self.item(first))]),
            );
        }
        tips.extend(self.list_line("notify.follow_up", &brief.needs_follow_up));
        (summary, tips)
    }

    /// The counts and the tips of the evening.
    fn evening_lines(&self, brief: &DailyBrief) -> (String, Vec<String>) {
        let tips = [
            self.list_line("notify.tomorrow", &brief.due_tomorrow()),
            self.list_line("notify.follow_up", &brief.needs_follow_up),
        ];
        (self.counts(brief), tips.into_iter().flatten().collect())
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
