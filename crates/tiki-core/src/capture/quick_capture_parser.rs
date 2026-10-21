use chrono::NaiveDate;

use super::{CaptureDraft, DatePhraseMatcher};
use crate::domain::{Client, ClientId};
use crate::error::{TikiError, TikiResult};
use crate::i18n::{CaptureVocabulary, TextNormalizer};

/// Reads "probar saldo vacacional andeanvet viernes" as
/// title "Probar saldo vacacional", client Andeanvet, due next Friday.
pub struct QuickCaptureParser {
    date_matcher: DatePhraseMatcher,
    connector_words: Vec<String>,
    known_clients: Vec<KnownClient>,
}

impl QuickCaptureParser {
    pub fn new(vocabulary: &CaptureVocabulary, clients: &[Client]) -> Self {
        Self {
            date_matcher: DatePhraseMatcher::new(vocabulary),
            connector_words: vocabulary
                .connector_words
                .iter()
                .map(|word| TextNormalizer::normalize(word))
                .collect(),
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

        for word in sentence.split_whitespace() {
            let normalized = TextNormalizer::normalize(word);
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
        let title = Self::capitalize(&title_words.join(" "));
        if title.is_empty() {
            return Err(TikiError::EmptyTaskTitle);
        }
        Ok(CaptureDraft {
            title,
            client_id: client.map(|known_client| known_client.id),
            client_name: client.map(|known_client| known_client.name.clone()),
            due_date,
        })
    }

    fn client_named(&self, normalized_word: &str) -> Option<&KnownClient> {
        self.known_clients
            .iter()
            .find(|known_client| known_client.normalized_name == normalized_word)
    }

    /// Removes words like "para" or "by" left dangling before a date.
    fn drop_trailing_connector(&self, title_words: &mut Vec<&str>) {
        while title_words.last().is_some_and(|word| {
            self.connector_words
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
