use serde::{Deserialize, Serialize};

use super::Speaker;

/// One line of a transcript, placed in the recorded time (pauses excluded).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub speaker: Speaker,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

/// What was said in a session, in the order it was said.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Transcript {
    segments: Vec<TranscriptSegment>,
}

impl Transcript {
    /// Joins the lines of every source by time; lines without words are left out.
    pub fn merge(sources: impl IntoIterator<Item = Vec<TranscriptSegment>>) -> Self {
        let mut segments: Vec<TranscriptSegment> = sources
            .into_iter()
            .flatten()
            .map(|segment| TranscriptSegment {
                text: segment.text.trim().to_owned(),
                ..segment
            })
            .filter(|segment| !segment.text.is_empty())
            .collect();
        segments.sort_by_key(|segment| (segment.start_ms, segment.speaker, segment.end_ms));
        Self { segments }
    }

    pub fn segments(&self) -> &[TranscriptSegment] {
        &self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}
