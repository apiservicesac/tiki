//! Fakes for the recording tests: scripted sources, a clock moved by hand, a media sink
//! that keeps what it receives, and a folder removed after the test.
//!
//! Each test file uses a different part of this module, so unused items are expected.
#![allow(dead_code)]

pub mod model_server;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tiki_recording::RecordingResult;
use tiki_recording::audio::AudioSource;
use tiki_recording::media::MediaSink;
use tiki_recording::pipeline::MonotonicClock;

/// A source fed by the test: what is pushed arrives at the next drain.
#[derive(Clone, Default)]
pub struct FakeSource {
    queued: Arc<Mutex<Vec<f32>>>,
}

impl FakeSource {
    pub fn push(&self, samples: &[f32]) {
        self.queued
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(samples);
    }
}

impl AudioSource for FakeSource {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn drain_into(&mut self, samples: &mut Vec<f32>) -> RecordingResult<()> {
        samples.append(&mut self.queued.lock().unwrap_or_else(PoisonError::into_inner));
        Ok(())
    }
}

/// A steady clock the test moves.
#[derive(Clone, Default)]
pub struct FakeClock {
    elapsed: Arc<Mutex<Duration>>,
}

impl FakeClock {
    pub fn advance(&self, by: Duration) {
        *self.elapsed.lock().unwrap_or_else(PoisonError::into_inner) += by;
    }
}

impl MonotonicClock for FakeClock {
    fn elapsed(&self) -> Duration {
        *self.elapsed.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Keeps every mixed sample it receives.
#[derive(Clone, Default)]
pub struct FakeMediaSink {
    pub written: Arc<Mutex<Vec<f32>>>,
}

impl FakeMediaSink {
    pub fn samples(&self) -> Vec<f32> {
        self.written
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl MediaSink for FakeMediaSink {
    fn write_audio(&mut self, mixed: &[f32]) -> RecordingResult<()> {
        self.written
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(mixed);
        Ok(())
    }

    fn finish(self: Box<Self>) -> RecordingResult<u64> {
        Ok(0)
    }
}

/// A fresh folder under the system's temporary folder, removed when dropped.
pub struct TestFolder {
    path: PathBuf,
}

impl TestFolder {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "tiki-recording-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("temporary folder");
        Self { path }
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
