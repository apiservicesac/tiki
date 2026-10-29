use std::time::Duration;

use super::{MonotonicClock, PauseClock};
use crate::audio::{AudioMixer, AudioSource, SourceAligner};
use crate::error::RecordingResult;
use crate::media::{MediaSink, TrackWriter};
use crate::recording_config::RecordingConfig;

/// The two sources of a session and the files they go to.
pub struct RecordingTracks {
    pub microphone: Box<dyn AudioSource>,
    pub system: Box<dyn AudioSource>,
    pub microphone_track: TrackWriter,
    pub system_track: TrackWriter,
    pub media: Box<dyn MediaSink>,
}

/// Moves audio from the sources to the files. Each pump takes exactly the samples the
/// recording clock says are due from both sources (filled with silence or trimmed by their
/// aligners), writes each one to its track and the mix to the media. While paused, what
/// the sources capture is thrown away and the timeline stands still.
pub struct RecordingPipeline {
    tracks: RecordingTracks,
    clock: Box<dyn MonotonicClock>,
    pause_clock: PauseClock,
    microphone_aligner: SourceAligner,
    system_aligner: SourceAligner,
    mixer: AudioMixer,
    written_samples: u64,
}

impl RecordingPipeline {
    const MICROS_PER_SECOND: u128 = 1_000_000;

    pub fn new(tracks: RecordingTracks, clock: Box<dyn MonotonicClock>) -> Self {
        let max_backlog = Self::samples_in(RecordingConfig::MAX_SOURCE_BACKLOG);
        Self {
            tracks,
            clock,
            pause_clock: PauseClock::default(),
            microphone_aligner: SourceAligner::new(Self::index(max_backlog)),
            system_aligner: SourceAligner::new(Self::index(max_backlog)),
            mixer: AudioMixer::default(),
            written_samples: 0,
        }
    }

    /// Writes what is due now; call it every `RecordingConfig::PUMP_INTERVAL`.
    pub fn pump(&mut self) -> RecordingResult<()> {
        if self.pause_clock.is_paused() {
            self.microphone_aligner
                .discard(self.tracks.microphone.as_mut())?;
            return self.system_aligner.discard(self.tracks.system.as_mut());
        }
        self.microphone_aligner
            .receive(self.tracks.microphone.as_mut())?;
        self.system_aligner.receive(self.tracks.system.as_mut())?;
        let due = Self::samples_in(self.recorded()).saturating_sub(self.written_samples);
        if due == 0 {
            return Ok(());
        }
        let microphone = self.microphone_aligner.take(Self::index(due));
        let system = self.system_aligner.take(Self::index(due));
        self.tracks.microphone_track.write(&microphone)?;
        self.tracks.system_track.write(&system)?;
        self.tracks
            .media
            .write_audio(&self.mixer.mix(&microphone, &system))?;
        self.written_samples += due;
        Ok(())
    }

    /// Writes what was said up to now, then stops the timeline.
    pub fn pause(&mut self) -> RecordingResult<()> {
        self.pump()?;
        self.pause_clock.pause(self.clock.elapsed());
        Ok(())
    }

    /// The timeline goes on from where it stopped; the pause leaves no gap.
    pub fn resume(&mut self) -> RecordingResult<()> {
        self.pump()?;
        self.pause_clock.resume(self.clock.elapsed());
        Ok(())
    }

    /// Time on the media timeline (pauses excluded).
    pub fn recorded(&self) -> Duration {
        self.pause_clock.recorded(self.clock.elapsed())
    }

    /// Writes the rest and closes every file; returns the bytes the media takes.
    pub fn finish(mut self) -> RecordingResult<u64> {
        self.pump()?;
        self.tracks.microphone_track.finish()?;
        self.tracks.system_track.finish()?;
        self.tracks.media.finish()
    }

    /// Samples at the mix rate that `duration` holds.
    fn samples_in(duration: Duration) -> u64 {
        let samples = duration.as_micros() * u128::from(RecordingConfig::MIX_SAMPLE_RATE)
            / Self::MICROS_PER_SECOND;
        u64::try_from(samples).unwrap_or(u64::MAX)
    }

    fn index(samples: u64) -> usize {
        usize::try_from(samples).unwrap_or(usize::MAX)
    }
}
