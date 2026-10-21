use super::BriefNotification;
use crate::briefing::{BriefDetail, BriefItem, BriefMoment, DailyBrief};
use crate::i18n::Translator;

/// Writes the short notification for a brief: counts first, then what to start with and
/// who to follow up.
pub struct NotificationComposer<'translator> {
    translator: &'translator Translator,
}

impl<'translator> NotificationComposer<'translator> {
    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    pub fn compose(&self, brief: &DailyBrief, moment: BriefMoment) -> BriefNotification {
        let lines = match moment {
            BriefMoment::Morning => self.morning_lines(brief),
            BriefMoment::Evening => self.evening_lines(brief),
        };
        BriefNotification {
            moment,
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
        let tomorrow: Vec<BriefItem> = brief
            .due_soon
            .iter()
            .filter(|item| item.detail == BriefDetail::DueIn { days: 1 })
            .cloned()
            .collect();
        let mut lines = vec![self.counts(brief)];
        lines.extend(self.list_line("notify.tomorrow", &tomorrow));
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
        match &item.client_name {
            Some(client) => self.translator.format(
                "narration.item_with_client",
                &[("title", &item.title), ("client", client)],
            ),
            None => item.title.clone(),
        }
    }
}
