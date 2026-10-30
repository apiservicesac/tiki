use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// Where a session stands, from the click that starts it to its transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Recording,
    Paused,
    /// Stopped: the media is saved and waits in the transcription queue.
    Queued,
    Transcribing,
    /// The transcript is ready.
    Ready,
    /// The transcription failed; the user can queue it again.
    Failed,
}

impl SessionStatus {
    pub const ALL: [SessionStatus; 6] = [
        SessionStatus::Recording,
        SessionStatus::Paused,
        SessionStatus::Queued,
        SessionStatus::Transcribing,
        SessionStatus::Ready,
        SessionStatus::Failed,
    ];

    /// Every move a session can make, from one status to the next.
    const TRANSITIONS: [(SessionStatus, SessionStatus); 9] = [
        (SessionStatus::Recording, SessionStatus::Paused),
        (SessionStatus::Paused, SessionStatus::Recording),
        (SessionStatus::Recording, SessionStatus::Queued),
        (SessionStatus::Paused, SessionStatus::Queued),
        (SessionStatus::Queued, SessionStatus::Transcribing),
        (SessionStatus::Transcribing, SessionStatus::Ready),
        (SessionStatus::Transcribing, SessionStatus::Failed),
        // Tiki closed while transcribing: the job goes back to the queue.
        (SessionStatus::Transcribing, SessionStatus::Queued),
        (SessionStatus::Failed, SessionStatus::Queued),
    ];

    /// Stable key used in storage and in translation keys (`session.status.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            SessionStatus::Recording => "recording",
            SessionStatus::Paused => "paused",
            SessionStatus::Queued => "queued",
            SessionStatus::Transcribing => "transcribing",
            SessionStatus::Ready => "ready",
            SessionStatus::Failed => "failed",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "session status",
                value: key.to_owned(),
            })
    }

    /// Whether the microphone is still taken: recording, or paused.
    pub fn is_capturing(self) -> bool {
        matches!(self, SessionStatus::Recording | SessionStatus::Paused)
    }

    /// The same status, or the next one if the move is allowed.
    pub fn checked_move(self, to: SessionStatus) -> TikiResult<SessionStatus> {
        if Self::TRANSITIONS.contains(&(self, to)) {
            Ok(to)
        } else {
            Err(TikiError::SessionTransition {
                from: self.key(),
                to: to.key(),
            })
        }
    }
}
