mod support;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;

use support::TestFolder;
use tiki_core::sessions::{SessionId, Speaker, Transcript};
use tiki_recording::media::TrackWriter;
use tiki_recording::transcription::{
    TimedText, TranscribedAudio, Transcriber, TranscriptionJob, TranscriptionQueue,
    TranscriptionWorker,
};
use tiki_recording::{RecordingConfig, RecordingError, RecordingResult};

const ONE_SECOND: usize = RecordingConfig::MIX_SAMPLE_RATE as usize;
const SPEECH: f32 = 0.5;

/// The core's queue, in memory: it hands out jobs and keeps what the worker reported.
#[derive(Default)]
struct FakeQueue {
    waiting: RefCell<VecDeque<TranscriptionJob>>,
    completed: RefCell<Vec<(SessionId, Transcript, Option<String>)>>,
    failed: RefCell<Vec<(SessionId, String)>>,
}

impl TranscriptionQueue for FakeQueue {
    fn next(&self) -> RecordingResult<Option<TranscriptionJob>> {
        Ok(self.waiting.borrow_mut().pop_front())
    }

    fn complete(
        &self,
        job: &TranscriptionJob,
        transcript: &Transcript,
        language: Option<String>,
    ) -> RecordingResult<()> {
        self.completed
            .borrow_mut()
            .push((job.session_id, transcript.clone(), language));
        Ok(())
    }

    fn fail(&self, job: &TranscriptionJob, reason: String) -> RecordingResult<()> {
        self.failed.borrow_mut().push((job.session_id, reason));
        Ok(())
    }
}

/// Hears the given audio for each track in turn, or fails when told to.
struct FakeTranscriber {
    answers: VecDeque<RecordingResult<TranscribedAudio>>,
    calls: usize,
}

impl Transcriber for FakeTranscriber {
    fn transcribe(&mut self, _samples: &[f32]) -> RecordingResult<TranscribedAudio> {
        self.calls += 1;
        self.answers
            .pop_front()
            .unwrap_or_else(|| Ok(TranscribedAudio::default()))
    }
}

fn heard(language: &str, lines: &[(i64, i64, &str)]) -> TranscribedAudio {
    TranscribedAudio {
        language: Some(language.to_owned()),
        segments: lines
            .iter()
            .map(|(start_ms, end_ms, text)| TimedText {
                start_ms: *start_ms,
                end_ms: *end_ms,
                text: (*text).to_owned(),
            })
            .collect(),
    }
}

fn write_track(folder: &Path, file: &str, level: f32) {
    let mut track = TrackWriter::create(&folder.join(file)).unwrap();
    track.write(&vec![level; ONE_SECOND]).unwrap();
    track.finish().unwrap();
}

fn queued_session(folder: &TestFolder, microphone: f32, system: f32) -> FakeQueue {
    write_track(
        folder.path(),
        RecordingConfig::MICROPHONE_TRACK_FILE,
        microphone,
    );
    write_track(folder.path(), RecordingConfig::SYSTEM_TRACK_FILE, system);
    let queue = FakeQueue::default();
    queue.waiting.borrow_mut().push_back(TranscriptionJob {
        session_id: SessionId(7),
        folder: folder.path().to_path_buf(),
    });
    queue
}

#[test]
fn the_microphone_is_you_and_the_system_audio_the_others_merged_by_time() {
    let folder = TestFolder::new();
    let queue = queued_session(&folder, SPEECH, SPEECH);
    let mut transcriber = FakeTranscriber {
        answers: VecDeque::from([
            Ok(heard("es", &[(0, 400, "Hola"), (900, 1_000, "¿Me oyen?")])),
            Ok(heard("es", &[(500, 800, "Buenas tardes, empezamos")])),
        ]),
        calls: 0,
    };

    assert!(
        TranscriptionWorker::new(&queue)
            .run_next(&mut transcriber)
            .unwrap()
    );

    let completed = queue.completed.borrow();
    let (session_id, transcript, language) = &completed[0];
    assert_eq!(*session_id, SessionId(7));
    assert_eq!(language.as_deref(), Some("es"));
    let lines: Vec<(Speaker, &str)> = transcript
        .segments()
        .iter()
        .map(|segment| (segment.speaker, segment.text.as_str()))
        .collect();
    assert_eq!(
        lines,
        [
            (Speaker::You, "Hola"),
            (Speaker::Others, "Buenas tardes, empezamos"),
            (Speaker::You, "¿Me oyen?")
        ]
    );
    assert!(!folder.file(RecordingConfig::MICROPHONE_TRACK_FILE).exists());
    assert!(!folder.file(RecordingConfig::SYSTEM_TRACK_FILE).exists());
}

#[test]
fn a_silent_track_is_not_given_to_whisper() {
    let folder = TestFolder::new();
    let queue = queued_session(&folder, SPEECH, 0.0);
    let mut transcriber = FakeTranscriber {
        answers: VecDeque::from([Ok(heard("en", &[(0, 300, "Hello")]))]),
        calls: 0,
    };

    TranscriptionWorker::new(&queue)
        .run_next(&mut transcriber)
        .unwrap();

    assert_eq!(transcriber.calls, 1);
    assert_eq!(queue.completed.borrow()[0].1.segments().len(), 1);
}

#[test]
fn a_failed_transcription_goes_back_to_the_queue_with_its_tracks() {
    let folder = TestFolder::new();
    let queue = queued_session(&folder, SPEECH, SPEECH);
    let mut transcriber = FakeTranscriber {
        answers: VecDeque::from([Err(RecordingError::Transcription("model".to_owned()))]),
        calls: 0,
    };

    TranscriptionWorker::new(&queue)
        .run_next(&mut transcriber)
        .unwrap();

    assert_eq!(queue.failed.borrow()[0].0, SessionId(7));
    assert!(queue.completed.borrow().is_empty());
    assert!(folder.file(RecordingConfig::MICROPHONE_TRACK_FILE).exists());
}

#[test]
fn an_empty_queue_has_nothing_to_do() {
    let queue = FakeQueue::default();
    let mut transcriber = FakeTranscriber {
        answers: VecDeque::new(),
        calls: 0,
    };
    assert!(
        !TranscriptionWorker::new(&queue)
            .run_next(&mut transcriber)
            .unwrap()
    );
}
