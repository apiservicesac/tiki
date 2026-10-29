//! Recorded sessions: a call or a meeting the user chose to record, its local transcript
//! and the notes the AI wrote about it. Nothing is recorded without the user's click.

mod active_recording;
mod identifiers;
mod meeting_link;
mod recording_mode;
mod recording_suggestion_policy;
mod retention_policy;
mod session;
mod session_clock;
mod session_notes;
mod session_search_hit;
mod session_service;
mod session_status;
mod session_summary;
mod speaker;
mod transcript;
mod transcript_query;

pub use active_recording::ActiveRecording;
pub use identifiers::{ActionItemId, SessionId};
pub use meeting_link::MeetingLink;
pub use recording_mode::RecordingMode;
pub use recording_suggestion_policy::{PresenceSignals, RecordingCue, RecordingSuggestionPolicy};
pub use retention_policy::RetentionPolicy;
pub use session::Session;
pub use session_clock::SessionClock;
pub use session_notes::{ProposedSessionTask, SessionActionItem, SessionNotes};
pub use session_search_hit::SessionSearchHit;
pub use session_service::SessionService;
pub use session_status::SessionStatus;
pub use session_summary::{SessionDetails, SessionSummary};
pub use speaker::Speaker;
pub use transcript::{Transcript, TranscriptSegment};
pub use transcript_query::TranscriptQuery;
