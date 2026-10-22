use super::{IslandSnapshot, SnapshotItem, SnapshotSection, Urgency};
use crate::briefing::{BriefDetailFormatter, BriefItem, BriefMoment, BriefNarrator, DailyBrief};
use crate::domain::ProjectDirectory;
use crate::i18n::{DateFormatter, Translator};
use crate::mood::Mood;

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
    ) -> IslandSnapshot {
        let sections = [
            ("overdue", &brief.overdue, Urgency::Critical),
            ("due_soon", &brief.due_soon, Urgency::Soon),
            ("your_turn", &brief.your_turn, Urgency::Normal),
            ("waiting", &brief.waiting, Urgency::Waiting),
            ("reminders", &brief.reminders, Urgency::Reminder),
        ]
        .into_iter()
        .filter(|(_, items, _)| !items.is_empty())
        .map(|(key, items, urgency)| self.section(key, items, urgency, brief))
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
        let detail_formatter = BriefDetailFormatter::new(self.translator);
        SnapshotSection {
            key: key.to_owned(),
            title: self.translator.text(&format!("brief.section.{key}")),
            items: items
                .iter()
                .map(|item| SnapshotItem {
                    task_id: item.task_id,
                    title: item.title.clone(),
                    project_name: item.project_name.clone(),
                    project_color: self.project_color(item),
                    detail_label: detail_formatter.label(item.detail, brief.date),
                    tags: item.tags.clone(),
                    urgency,
                })
                .collect(),
        }
    }

    fn project_color(&self, item: &BriefItem) -> Option<String> {
        let name = item.project_name.as_deref()?;
        self.projects
            .projects()
            .find(|project| project.name == name)
            .map(|project| project.color.clone())
    }
}
