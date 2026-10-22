use super::{BriefItem, BriefMoment, DailyBrief};
use crate::i18n::{DateFormatter, Translator};

/// Turns a brief into the sentence Tiki reads out loud or shows in a notification.
pub struct BriefNarrator<'translator> {
    translator: &'translator Translator,
}

impl<'translator> BriefNarrator<'translator> {
    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    pub fn narrate(&self, brief: &DailyBrief, moment: BriefMoment) -> String {
        let date = DateFormatter::new(self.translator).long(brief.date);
        let mut sentences = vec![
            self.translator
                .text(&format!("narration.greeting.{}", moment.key())),
            self.translator
                .format("narration.today_is", &[("date", &date)]),
        ];

        if brief.is_all_clear() {
            sentences.push(self.translator.text("narration.all_clear"));
        }
        sentences.extend(self.section("narration.overdue", &brief.overdue));
        sentences.extend(self.section("narration.due_soon", &brief.due_soon));
        sentences.extend(self.section("narration.your_turn", &brief.your_turn));
        sentences.extend(self.section("narration.waiting", &brief.waiting));
        sentences.join(" ")
    }

    /// One sentence for a non-empty section, nothing for an empty one.
    fn section(&self, key: &str, items: &[BriefItem]) -> Option<String> {
        if items.is_empty() {
            return None;
        }
        let spoken_items: Vec<String> = items.iter().map(|item| self.spoken_item(item)).collect();
        let list = self.translator.join_list(&spoken_items);
        Some(self.translator.format(key, &[("items", &list)]))
    }

    fn spoken_item(&self, item: &BriefItem) -> String {
        match &item.project_name {
            Some(project) => self.translator.format(
                "narration.item_with_project",
                &[("title", &item.title), ("project", project)],
            ),
            None => item.title.clone(),
        }
    }
}
