use chrono::NaiveDate;

use super::{CaptureIntent, QuickCaptureParser};
use crate::error::{TikiError, TikiResult};

/// Decides whether a sentence creates a client ("+Andeanvet") or a task (anything else).
pub struct CaptureIntentReader<'parser> {
    task_parser: &'parser QuickCaptureParser,
}

impl<'parser> CaptureIntentReader<'parser> {
    /// Marks a sentence as a new client in every language.
    pub const NEW_CLIENT_PREFIX: char = '+';

    pub fn new(task_parser: &'parser QuickCaptureParser) -> Self {
        Self { task_parser }
    }

    pub fn read(&self, sentence: &str, today: NaiveDate) -> TikiResult<CaptureIntent> {
        match sentence.trim().strip_prefix(Self::NEW_CLIENT_PREFIX) {
            Some(client_name) => {
                let name = client_name.trim();
                if name.is_empty() {
                    return Err(TikiError::EmptyClientName);
                }
                Ok(CaptureIntent::Client {
                    name: name.to_owned(),
                })
            }
            None => Ok(CaptureIntent::Task(
                self.task_parser.parse(sentence, today)?,
            )),
        }
    }
}
