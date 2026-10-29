use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use hound::{SampleFormat as WavSampleFormat, WavSpec, WavWriter};

use crate::audio::{Decimator, SampleFormat};
use crate::error::RecordingResult;
use crate::recording_config::RecordingConfig;

/// One source written as 16 kHz mono 16-bit WAV, the input of the transcription. Its header
/// is rewritten every `TRACK_FLUSH_INTERVAL`, so after a crash the file still reads up to
/// the last few seconds.
pub struct TrackWriter {
    writer: WavWriter<BufWriter<File>>,
    decimator: Decimator,
    unflushed_samples: u64,
    flush_every: u64,
}

impl TrackWriter {
    const BITS_PER_SAMPLE: u16 = 16;
    const MONO: u16 = 1;

    pub fn create(path: &Path) -> RecordingResult<Self> {
        let spec = WavSpec {
            channels: Self::MONO,
            sample_rate: RecordingConfig::TRANSCRIPTION_SAMPLE_RATE,
            bits_per_sample: Self::BITS_PER_SAMPLE,
            sample_format: WavSampleFormat::Int,
        };
        let flush_seconds = RecordingConfig::TRACK_FLUSH_INTERVAL.as_secs();
        Ok(Self {
            writer: WavWriter::create(path, spec)?,
            decimator: Decimator::default(),
            unflushed_samples: 0,
            flush_every: u64::from(RecordingConfig::TRANSCRIPTION_SAMPLE_RATE) * flush_seconds,
        })
    }

    /// Appends 48 kHz samples, decimated to 16 kHz.
    pub fn write(&mut self, samples: &[f32]) -> RecordingResult<()> {
        let decimated = self.decimator.process(samples);
        for sample in &decimated {
            self.writer.write_sample(SampleFormat::to_i16(*sample))?;
        }
        self.unflushed_samples += u64::try_from(decimated.len()).unwrap_or(u64::MAX);
        if self.unflushed_samples >= self.flush_every {
            self.writer.flush()?;
            self.unflushed_samples = 0;
        }
        Ok(())
    }

    pub fn finish(self) -> RecordingResult<()> {
        self.writer.finalize()?;
        Ok(())
    }
}
