use chrono::NaiveDateTime;

use super::{FocusSuggestion, IslandSnapshot, SnapshotItem, SnapshotSection, Urgency};
use crate::briefing::{
    BriefDetailFormatter, BriefItem, BriefMoment, BriefNarrator, DailyBrief, FocusPicker,
    FocusReason,
};
use crate::calendar::MeetingItem;
use crate::domain::ProjectDirectory;
use crate::i18n::{DateFormatter, Translator};
use crate::mood::Mood;

/// The calendar side of the snapshot: the Agenda's meetings and the next one today.
pub struct TodaysMeetings {
    pub now: NaiveDateTime,
    pub next: Option<MeetingItem>,
    pub agenda: Vec<MeetingItem>,
}

/// Translates a brief and a mood into an `IslandSnapshot`.
pub struct SnapshotPresenter<'context> {
    translator: &'context Translator,
    projects: &'context ProjectDirectory,
}

impl<'context> SnapshotPresenter<'context> {
    pub fn new(translator: &'context Translator, projects: &'context ProjectDirectory) -> Self {
        Self {
            translator,
            projects,
        }
    }

    pub fn present(
        &self,
        brief: &DailyBrief,
        mood: Mood,
        delivered_today: usize,
        meetings: &TodaysMeetings,
    ) -> IslandSnapshot {
        let todays_reminders = brief.reminders_on(brief.date);
        let sections = [
            ("overdue", &brief.overdue, Urgency::Critical),
            ("due_soon", &brief.due_soon, Urgency::Soon),
            ("reminders", &todays_reminders, Urgency::Reminder),
            ("follow_up", &brief.needs_follow_up, Urgency::Waiting),
        ]
        .into_iter()
        .filter(|(_, items, _)| !items.is_empty())
        .map(|(key, items, urgency)| self.section(key, items, urgency, brief))
        .collect();
        let open_items: Vec<SnapshotItem> = [
            (&brief.overdue, Urgency::Critical),
            (&brief.due_soon, Urgency::Soon),
            (&brief.your_turn, Urgency::Normal),
            (&brief.waiting, Urgency::Waiting),
            (&brief.reminders, Urgency::Reminder),
        ]
        .into_iter()
        .flat_map(|(items, urgency)| items.iter().map(move |item| (item, urgency)))
        .map(|(item, urgency)| self.item(item, urgency, brief))
        .collect();

        IslandSnapshot {
            mood,
            mood_label: self.translator.text(&format!("mood.label.{}", mood.key())),
            attention_count: brief.pressing_count() + brief.waiting.len(),
            due_count: brief.pressing_count(),
            waiting_count: brief.waiting.len(),
            delivered_today,
            closed_summary: self.translator.format(
                "island.closed.summary",
                &[
                    ("due", &brief.pressing_count().to_string()),
                    ("waiting", &brief.waiting.len().to_string()),
                ],
            ),
            today_subtitle: self.translator.format(
                "island.today.subtitle",
                &[
                    (
                        "date",
                        &DateFormatter::new(self.translator).long(brief.date),
                    ),
                    ("count", &brief.pressing_count().to_string()),
                ],
            ),
            sections,
            open_items,
            focus: self.focus(brief, meetings),
            meetings: meetings.agenda.clone(),
            narration: BriefNarrator::new(self.translator).narrate(brief, BriefMoment::Morning),
        }
    }

    fn section(
        &self,
        key: &str,
        items: &[BriefItem],
        urgency: Urgency,
        brief: &DailyBrief,
    ) -> SnapshotSection {
        SnapshotSection {
            key: key.to_owned(),
            title: self.translator.text(&format!("brief.section.{key}")),
            items: items
                .iter()
                .map(|item| self.item(item, urgency, brief))
                .collect(),
        }
    }

    fn item(&self, item: &BriefItem, urgency: Urgency, brief: &DailyBrief) -> SnapshotItem {
        SnapshotItem {
            task_id: item.task_id,
            kind: item.kind,
            title: item.title.clone(),
            project_id: item.project_id,
            project_name: item.project_name.clone(),
            project_color: self.project_color(item),
            detail_label: BriefDetailFormatter::new(self.translator).label(item.detail, brief.date),
            priority: item.priority,
            tags: item.tags.clone(),
            at: item.rings_at(),
            repeat: item.repeat(),
            urgency,
        }
    }

    /// What to do now (see `FocusPicker`), with the free time before the next meeting.
    fn focus(&self, brief: &DailyBrief, meetings: &TodaysMeetings) -> Option<FocusSuggestion> {
        let mut focus = self.focus_item(brief)?;
        if let Some(next) = &meetings.next {
            let minutes = (next.starts_at - meetings.now).num_minutes();
            let free = self.translator.format(
                "focus.free_time",
                &[("minutes", &minutes.to_string()), ("meeting", &next.title)],
            );
            focus.reason = format!("{} · {free}", focus.reason);
        }
        Some(focus)
    }

    fn focus_item(&self, brief: &DailyBrief) -> Option<FocusSuggestion> {
        let pick = FocusPicker::pick(brief)?;
        let urgency = match pick.reason {
            FocusReason::Overdue => Urgency::Critical,
            FocusReason::DueToday | FocusReason::DueSoon => Urgency::Soon,
            FocusReason::Important => Urgency::Normal,
            FocusReason::FollowUp => Urgency::Waiting,
        };
        Some(FocusSuggestion {
            item: self.item(pick.item, urgency, brief),
            reason: self
                .translator
                .text(&format!("focus.{}", pick.reason.key())),
        })
    }

    fn project_color(&self, item: &BriefItem) -> Option<String> {
        let project_id = item.project_id?;
        self.projects
            .find(project_id)
            .map(|project| project.color.clone())
    }
}
