mod support;

use std::time::Duration;

use support::{FakeClock, FakeMediaSink, FakeSource, TestFolder};
use tiki_recording::RecordingConfig;
use tiki_recording::media::TrackWriter;
use tiki_recording::pipeline::{RecordingPipeline, RecordingTracks};

const RATE: usize = RecordingConfig::MIX_SAMPLE_RATE as usize;
const TRACK_RATE: usize = RecordingConfig::TRANSCRIPTION_SAMPLE_RATE as usize;
const SPEECH: f32 = 0.5;
const SAID_DURING_PAUSE: f32 = 0.9;

/// A pipeline over fake sources, a fake clock and a sink that keeps the mix.
struct Rig {
    microphone: FakeSource,
    system: FakeSource,
    clock: FakeClock,
    media: FakeMediaSink,
    pipeline: RecordingPipeline,
    folder: TestFolder,
}

impl Rig {
    fn new() -> Self {
        let folder = TestFolder::new();
        let microphone = FakeSource::default();
        let system = FakeSource::default();
        let clock = FakeClock::default();
        let media = FakeMediaSink::default();
        let pipeline = RecordingPipeline::new(
            RecordingTracks {
                microphone: Box::new(microphone.clone()),
                system: Box::new(system.clone()),
                microphone_track: TrackWriter::create(
                    &folder.file(RecordingConfig::MICROPHONE_TRACK_FILE),
                )
                .unwrap(),
                system_track: TrackWriter::create(&folder.file(RecordingConfig::SYSTEM_TRACK_FILE))
                    .unwrap(),
                media: Box::new(media.clone()),
            },
            Box::new(clock.clone()),
        );
        Self {
            microphone,
            system,
            clock,
            media,
            pipeline,
            folder,
        }
    }

    /// One second passes with `samples` captured by the microphone, then the pipeline pumps.
    fn second(&mut self, microphone: &[f32], system: &[f32]) {
        self.microphone.push(microphone);
        self.system.push(system);
        self.clock.advance(Duration::from_secs(1));
        self.pipeline.pump().unwrap();
    }
}

/// Samples a finished WAV track holds.
fn track_length(folder: &TestFolder, file: &str) -> usize {
    let reader = hound::WavReader::open(folder.file(file)).unwrap();
    usize::try_from(reader.duration()).unwrap()
}

fn second_of(level: f32) -> Vec<f32> {
    vec![level; RATE]
}

#[test]
fn a_silent_source_is_filled_with_silence_so_both_stay_in_step() {
    let mut rig = Rig::new();
    rig.second(&second_of(SPEECH), &[]);
    let mixed = rig.media.samples();
    assert_eq!(mixed.len(), RATE);
    assert!(
        mixed
            .iter()
            .all(|sample| (*sample - SPEECH).abs() < f32::EPSILON)
    );
    let Rig {
        pipeline, folder, ..
    } = rig;
    pipeline.finish().unwrap();
    assert_eq!(
        track_length(&folder, RecordingConfig::MICROPHONE_TRACK_FILE),
        TRACK_RATE
    );
    assert_eq!(
        track_length(&folder, RecordingConfig::SYSTEM_TRACK_FILE),
        TRACK_RATE
    );
}

#[test]
fn a_source_running_ahead_loses_its_oldest_surplus() {
    let mut rig = Rig::new();
    let backlog =
        RATE * usize::try_from(RecordingConfig::MAX_SOURCE_BACKLOG.as_millis()).unwrap() / 1_000;
    rig.second(&[], &[second_of(SPEECH), second_of(SPEECH)].concat());
    rig.second(&[], &[]);
    let mixed = rig.media.samples();
    assert_eq!(mixed.len(), 2 * RATE);
    assert!((mixed[RATE + backlog - 1] - SPEECH).abs() < f32::EPSILON);
    assert!(mixed[RATE + backlog].abs() < f32::EPSILON);
}

#[test]
fn nothing_said_during_a_pause_is_written_and_the_timeline_does_not_jump() {
    let mut rig = Rig::new();
    rig.second(&second_of(SPEECH), &[]);
    rig.pipeline.pause().unwrap();
    for _ in 0..5 {
        rig.second(&second_of(SAID_DURING_PAUSE), &second_of(SAID_DURING_PAUSE));
    }
    rig.pipeline.resume().unwrap();
    rig.second(&second_of(SPEECH), &[]);
    assert_eq!(rig.pipeline.recorded(), Duration::from_secs(2));

    let mixed = rig.media.samples();
    assert_eq!(mixed.len(), 2 * RATE);
    assert!(
        mixed
            .iter()
            .all(|sample| (*sample - SPEECH).abs() < f32::EPSILON)
    );
}

#[test]
fn the_tracks_survive_a_crash_up_to_their_last_flush() {
    let mut rig = Rig::new();
    let flushed_seconds = RecordingConfig::TRACK_FLUSH_INTERVAL.as_secs() + 1;
    for _ in 0..flushed_seconds {
        rig.second(&second_of(SPEECH), &[]);
    }
    // Tiki dies: nothing is finished or dropped properly.
    std::mem::forget(rig.pipeline);
    let reader =
        hound::WavReader::open(rig.folder.file(RecordingConfig::MICROPHONE_TRACK_FILE)).unwrap();
    let flush_samples =
        TRACK_RATE * usize::try_from(RecordingConfig::TRACK_FLUSH_INTERVAL.as_secs()).unwrap();
    assert!(usize::try_from(reader.duration()).unwrap() >= flush_samples);
}
