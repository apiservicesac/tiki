use std::path::PathBuf;

use tiki_core::sessions::{SessionId, Speaker, Transcript, TranscriptSegment};

use super::{TrackReader, TranscribedAudio, Transcriber};
use crate::error::RecordingResult;
use crate::recording_config::RecordingConfig;

/// A session waiting for its transcript, and the folder with its tracks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptionJob {
    pub session_id: SessionId,
    pub folder: PathBuf,
}

/// The persistent queue of sessions to transcribe (the core's `transcription_jobs`).
pub trait TranscriptionQueue {
    /// The oldest session waiting, now marked as being transcribed.
    fn next(&self) -> RecordingResult<Option<TranscriptionJob>>;

    fn complete(
        &self,
        job: &TranscriptionJob,
        transcript: &Transcript,
        language: Option<String>,
    ) -> RecordingResult<()>;

    /// The transcription failed; the queue decides whether to retry it.
    fn fail(&self, job: &TranscriptionJob, reason: String) -> RecordingResult<()>;
}

/// Transcribes the sessions in the queue, one at a time: the microphone track as "You" and
/// the system track as "Others", merged by time. Once a transcript is saved, the tracks are
/// removed; the media played back stays.
pub struct TranscriptionWorker<'queue> {
    queue: &'queue dyn TranscriptionQueue,
}

impl<'queue> TranscriptionWorker<'queue> {
    const TRACKS: [(&'static str, Speaker); 2] = [
        (RecordingConfig::MICROPHONE_TRACK_FILE, Speaker::You),
        (RecordingConfig::SYSTEM_TRACK_FILE, Speaker::Others),
    ];

    pub fn new(queue: &'queue dyn TranscriptionQueue) -> Self {
        Self { queue }
    }

    /// Transcribes the next session in the queue; `false` when the queue is empty.
    pub fn run_next(&self, transcriber: &mut dyn Transcriber) -> RecordingResult<bool> {
        let Some(job) = self.queue.next()? else {
            return Ok(false);
        };
        match Self::transcribe(&job, transcriber) {
            Ok((transcript, language)) => {
                self.queue.complete(&job, &transcript, language)?;
                for (file, _) in Self::TRACKS {
                    std::fs::remove_file(job.folder.join(file))?;
                }
            }
            Err(error) => self.queue.fail(&job, error.to_string())?,
        }
        Ok(true)
    }

    fn transcribe(
        job: &TranscriptionJob,
        transcriber: &mut dyn Transcriber,
    ) -> RecordingResult<(Transcript, Option<String>)> {
        let mut sources = Vec::new();
        let mut heard_most: Option<TranscribedAudio> = None;
        for (file, speaker) in Self::TRACKS {
            let samples = TrackReader::read(&job.folder.join(file))?;
            let heard = if Self::is_silent(&samples) {
                TranscribedAudio::default()
            } else {
                transcriber.transcribe(&samples)?
            };
            sources.push(
                heard
                    .segments
                    .iter()
                    .map(|segment| TranscriptSegment {
                        speaker,
                        start_ms: segment.start_ms,
                        end_ms: segment.end_ms,
                        text: segment.text.clone(),
                    })
                    .collect(),
            );
            if heard_most
                .as_ref()
                .is_none_or(|most| heard.spoken_ms() > most.spoken_ms())
            {
                heard_most = Some(heard);
            }
        }
        let language = heard_most.and_then(|heard| heard.language);
        Ok((Transcript::merge(sources), language))
    }

    fn is_silent(samples: &[f32]) -> bool {
        samples
            .iter()
            .all(|sample| sample.abs() < RecordingConfig::SILENT_TRACK_PEAK)
    }
}
