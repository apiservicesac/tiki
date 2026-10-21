use chrono::{NaiveDate, NaiveTime};

use super::{CaptureDraft, DatePhraseMatcher, TimePhraseMatcher};
use crate::domain::{Client, ClientId};
use crate::error::{TikiError, TikiResult};
use crate::i18n::{CaptureVocabulary, TextNormalizer};

/// Reads "probar saldo vacacional andeanvet viernes, avísame a las 9" as title "Probar
/// saldo vacacional", client Andeanvet, due next Friday, with a reminder that day at 09:00.
pub struct QuickCaptureParser {
    date_matcher: DatePhraseMatcher,
    time_matcher: TimePhraseMatcher,
    connector_words: Vec<String>,
    remind_words: Vec<String>,
    remind_fillers: Vec<String>,
    known_clients: Vec<KnownClient>,
    /// Used when the sentence asks for a reminder without saying the time.
    default_reminder_time: NaiveTime,
}

impl QuickCaptureParser {
    pub fn new(
        vocabulary: &CaptureVocabulary,
        clients: &[Client],
        default_reminder_time: NaiveTime,
    ) -> Self {
        let normalize_all = |words: &[String]| -> Vec<String> {
            words
                .iter()
                .map(|word| TextNormalizer::normalize(word))
                .collect()
        };
        Self {
            date_matcher: DatePhraseMatcher::new(vocabulary),
            time_matcher: TimePhraseMatcher::new(vocabulary),
            connector_words: normalize_all(&vocabulary.connector_words),
            remind_words: normalize_all(&vocabulary.remind_words),
            remind_fillers: normalize_all(&vocabulary.remind_fillers),
            default_reminder_time,
            known_clients: clients
                .iter()
                .filter_map(|client| {
                    client.id.map(|id| KnownClient {
                        id,
                        name: client.name.clone(),
                        normalized_name: TextNormalizer::normalize(&client.name),
                    })
                })
                .collect(),
        }
    }

    pub fn parse(&self, sentence: &str, today: NaiveDate) -> TikiResult<CaptureDraft> {
        let mut title_words: Vec<&str> = Vec::new();
        let mut client: Option<&KnownClient> = None;
        let mut due_date = None;
        let mut reminder_time: Option<NaiveTime> = None;
        let mut wants_reminder = false;
        let mut previous_normalized: Option<String> = None;

        for word in sentence.split_whitespace() {
            let normalized = TextNormalizer::normalize(word.trim_end_matches([',', ';', '.']));
            let previous = previous_normalized.replace(normalized.clone());
            if self.remind_words.contains(&normalized) {
                wants_reminder = true;
                self.drop_trailing_connector(&mut title_words);
                continue;
            }
            if wants_reminder && self.remind_fillers.contains(&normalized) {
                continue;
            }
            if reminder_time.is_none()
                && let Some(time) = self.time_matcher.resolve(&normalized, previous.as_deref())
            {
                reminder_time = Some(time);
                self.drop_trailing_connector(&mut title_words);
                continue;
            }
            if due_date.is_none()
                && let Some(date) = self.date_matcher.resolve(&normalized, today)
            {
                due_date = Some(date);
                self.drop_trailing_connector(&mut title_words);
                continue;
            }
            if client.is_none()
                && let Some(known_client) = self.client_named(&normalized)
            {
                client = Some(known_client);
                continue;
            }
            title_words.push(word);
        }

        self.drop_trailing_connector(&mut title_words);
        let title = Self::capitalize(title_words.join(" ").trim_end_matches([',', ';', '.']));
        if title.is_empty() {
            return Err(TikiError::EmptyTaskTitle);
        }
        Ok(CaptureDraft {
            title,
            client_id: client.map(|known_client| known_client.id),
            client_name: client.map(|known_client| known_client.name.clone()),
            due_date,
            remind_at: (wants_reminder || reminder_time.is_some()).then(|| {
                due_date
                    .unwrap_or(today)
                    .and_time(reminder_time.unwrap_or(self.default_reminder_time))
            }),
        })
    }

    fn client_named(&self, normalized_word: &str) -> Option<&KnownClient> {
        self.known_clients
            .iter()
            .find(|known_client| known_client.normalized_name == normalized_word)
    }

    /// Removes words like "para" or "by" left dangling before a date, and a trailing comma.
    fn drop_trailing_connector(&self, title_words: &mut Vec<&str>) {
        if let Some(last) = title_words.last_mut() {
            *last = last.trim_end_matches([',', ';']);
        }
        while title_words.last().is_some_and(|word| {
            word.is_empty()
                || self
                    .connector_words
                    .contains(&TextNormalizer::normalize(word))
        }) {
            title_words.pop();
        }
    }

    fn capitalize(text: &str) -> String {
        let mut characters = text.chars();
        characters.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(characters).collect()
        })
    }
}

/// A client the parser can recognize, with its name ready for comparison.
struct KnownClient {
    id: ClientId,
    name: String,
    normalized_name: String,
}
