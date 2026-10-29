use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// The time a session has recorded: from its start, minus every pause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionClock {
    pub started_at: NaiveDateTime,
    /// Paused time already over, in milliseconds.
    pub paused_ms: i64,
    /// When the pause in progress began; empty while recording.
    pub paused_since: Option<NaiveDateTime>,
}

impl SessionClock {
    pub fn start(at: NaiveDateTime) -> Self {
        Self {
            started_at: at,
            paused_ms: 0,
            paused_since: None,
        }
    }

    pub fn pause(&mut self, at: NaiveDateTime) {
        self.paused_since.get_or_insert(at);
    }

    pub fn resume(&mut self, at: NaiveDateTime) {
        if let Some(since) = self.paused_since.take() {
            self.paused_ms += (at - since).num_milliseconds().max(0);
        }
    }

    /// Recorded time at `at`: what the media lasts, and where a moment falls in it.
    pub fn recorded(&self, at: NaiveDateTime) -> Duration {
        let pause_in_progress = self
            .paused_since
            .map_or(0, |since| (at - since).num_milliseconds().max(0));
        let total = (at - self.started_at).num_milliseconds() - self.paused_ms - pause_in_progress;
        Duration::milliseconds(total.max(0))
    }
}
