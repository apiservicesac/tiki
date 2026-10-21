use super::{IslandSnapshot, SnapshotItem, SnapshotSection, Urgency};
use crate::briefing::{BriefDetailFormatter, BriefItem, BriefMoment, BriefNarrator, DailyBrief};
use crate::domain::ClientDirectory;
use crate::i18n::{DateFormatter, Translator};
use crate::mood::Mood;

/// Translates a brief and a mood into an `IslandSnapshot`.
pub struct SnapshotPresenter<'context> {
    translator: &'context Translator,
    clients: &'context ClientDirectory,
}

impl<'context> SnapshotPresenter<'context> {
    pub fn new(translator: &'context Translator, clients: &'context ClientDirectory) -> Self {
        Self {
            translator,
            clients,
        }
    }

    pub fn present(&self, brief: &DailyBrief, mood: Mood) -> IslandSnapshot {
        let sections = [
            ("overdue", &brief.overdue, Urgency::Critical),
            ("due_soon", &brief.due_soon, Urgency::Soon),
            ("your_turn", &brief.your_turn, Urgency::Normal),
            ("waiting", &brief.waiting, Urgency::Waiting),
        ]
        .into_iter()
        .filter(|(_, items, _)| !items.is_empty())
        .map(|(key, items, urgency)| self.section(key, items, urgency, brief))
        .collect();

        IslandSnapshot {
            mood,
            mood_label: self.translator.text(&format!("mood.label.{}", mood.key())),
            attention_count: brief.pressing_count() + brief.waiting.len(),
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
                    client_name: item.client_name.clone(),
                    client_color: self.client_color(item),
                    detail_label: detail_formatter.label(item.detail, brief.date),
                    urgency,
                })
                .collect(),
        }
    }

    fn client_color(&self, item: &BriefItem) -> Option<String> {
        let name = item.client_name.as_deref()?;
        self.clients
            .clients()
            .find(|client| client.name == name)
            .map(|client| client.color.clone())
    }
}
