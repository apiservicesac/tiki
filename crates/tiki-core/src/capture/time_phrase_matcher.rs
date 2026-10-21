use chrono::NaiveTime;

use crate::i18n::{CaptureVocabulary, TextNormalizer};

/// Recognizes a time of day: "a las 9", "9:30", "9am", "at 5pm", "15h".
///
/// A bare number counts only after a time marker ("a las", "at"), so "revisar 3 facturas"
/// keeps its 3. Without am/pm, hours from 1 to 7 mean the afternoon: nobody books work for
/// three in the morning.
pub struct TimePhraseMatcher {
    markers: Vec<String>,
}

impl TimePhraseMatcher {
    /// Before this hour, a bare hour means the afternoon ("a las 3" is 15:00).
    const EARLIEST_MORNING_HOUR: u32 = 8;
    const HOURS_TO_AFTERNOON: u32 = 12;

    pub fn new(vocabulary: &CaptureVocabulary) -> Self {
        Self {
            markers: vocabulary
                .time_markers
                .iter()
                .map(|word| TextNormalizer::normalize(word))
                .collect(),
        }
    }

    /// The time a normalized word means, looking at the word before it for a marker.
    pub fn resolve(&self, normalized_word: &str, previous_word: Option<&str>) -> Option<NaiveTime> {
        let word = normalized_word.trim_end_matches(['.', ',', ';']);
        let (number, suffix) = Self::split_suffix(word);
        let (hour_text, minute_text) = number.split_once(':').unwrap_or((number, "0"));
        let hour: u32 = hour_text.parse().ok()?;
        let minute: u32 = minute_text.parse().ok()?;

        let after_marker = previous_word
            .is_some_and(|previous| self.markers.iter().any(|marker| marker == previous));
        let looks_like_time = after_marker || number.contains(':') || !suffix.is_empty();
        if !looks_like_time {
            return None;
        }
        let hour = match suffix {
            "pm" if hour < Self::HOURS_TO_AFTERNOON => hour + Self::HOURS_TO_AFTERNOON,
            "am" if hour == Self::HOURS_TO_AFTERNOON => 0,
            "" if (1..Self::EARLIEST_MORNING_HOUR).contains(&hour) => {
                hour + Self::HOURS_TO_AFTERNOON
            }
            _ => hour,
        };
        NaiveTime::from_hms_opt(hour, minute, 0)
    }

    /// Splits "9pm" into ("9", "pm"); accepts am, pm and h (as in "15h").
    fn split_suffix(word: &str) -> (&str, &str) {
        for suffix in ["am", "pm", "hs", "h"] {
            if let Some(number) = word.strip_suffix(suffix) {
                return (number, if suffix.starts_with('h') { "h" } else { suffix });
            }
        }
        (word, "")
    }
}
