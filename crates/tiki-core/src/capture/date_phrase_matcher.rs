use chrono::{Datelike, Days, NaiveDate};

use crate::i18n::{CaptureVocabulary, TextNormalizer};

/// Recognizes single words that mean a day: "hoy", "mañana", "viernes", "friday".
pub struct DatePhraseMatcher {
    today: Vec<String>,
    tomorrow: Vec<String>,
    /// Accepted words per weekday, Monday first.
    weekdays: Vec<Vec<String>>,
}

impl DatePhraseMatcher {
    pub fn new(vocabulary: &CaptureVocabulary) -> Self {
        let normalize_all = |words: &[String]| {
            words
                .iter()
                .map(|word| TextNormalizer::normalize(word))
                .collect()
        };
        Self {
            today: normalize_all(&vocabulary.today),
            tomorrow: normalize_all(&vocabulary.tomorrow),
            weekdays: vocabulary
                .weekdays
                .iter()
                .map(|words| normalize_all(words))
                .collect(),
        }
    }

    /// The date a normalized word refers to, counted from `today`.
    ///
    /// A weekday means its next occurrence; naming today's weekday means next week.
    pub fn resolve(&self, normalized_word: &str, today: NaiveDate) -> Option<NaiveDate> {
        if self.today.iter().any(|word| word == normalized_word) {
            return Some(today);
        }
        if self.tomorrow.iter().any(|word| word == normalized_word) {
            return today.checked_add_days(Days::new(1));
        }
        let target_weekday = self
            .weekdays
            .iter()
            .position(|words| words.iter().any(|word| word == normalized_word))?;
        today.checked_add_days(Days::new(Self::days_until_weekday(today, target_weekday)))
    }

    fn days_until_weekday(today: NaiveDate, target_weekday: usize) -> u64 {
        let today_weekday = u64::from(today.weekday().num_days_from_monday());
        let target_weekday = target_weekday as u64;
        let days_ahead = (target_weekday + 7 - today_weekday) % 7;
        if days_ahead == 0 { 7 } else { days_ahead }
    }
}
