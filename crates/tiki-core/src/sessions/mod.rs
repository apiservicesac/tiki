//! Recorded sessions: a call or a meeting the user chose to record, its local transcript
//! and the notes the AI wrote about it. Nothing is recorded without the user's click.

mod identifiers;
mod meeting_link;
mod recording_mode;
mod retention_policy;
mod session;
mod session_clock;
mod session_notes;
mod session_search_hit;
mod session_status;
mod speaker;
mod transcript;
mod transcript_query;

pub use identifiers::{ActionItemId, SessionId};
pub use meeting_link::MeetingLink;
pub use recording_mode::RecordingMode;
pub use retention_policy::RetentionPolicy;
pub use session::Session;
pub use session_clock::SessionClock;
pub use session_notes::{ProposedSessionTask, SessionActionItem, SessionNotes};
pub use session_search_hit::SessionSearchHit;
pub use session_status::SessionStatus;
pub use speaker::Speaker;
pub use transcript::{Transcript, TranscriptSegment};
pub use transcript_query::TranscriptQuery;
