use std::collections::VecDeque;
use std::f32::consts::PI;

use crate::recording_config::RecordingConfig;

/// Brings 48 kHz audio down to 16 kHz for Whisper: a windowed-sinc low-pass filter, then one
/// sample of every three. It keeps the end of each block, so blocks join without clicks.
pub struct Decimator {
    taps: Vec<f32>,
    factor: usize,
    history: VecDeque<f32>,
    /// Samples received since the last one kept, so every block keeps the same phase.
    phase: usize,
}

impl Default for Decimator {
    fn default() -> Self {
        Self::new(
            RecordingConfig::MIX_SAMPLE_RATE,
            RecordingConfig::TRANSCRIPTION_SAMPLE_RATE,
        )
    }
}

impl Decimator {
    /// `from_rate` must be a whole multiple of `to_rate`.
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        let taps = Self::low_pass(
            RecordingConfig::DECIMATION_TAPS,
            RecordingConfig::DECIMATION_CUTOFF_HZ
                / Self::float(usize::try_from(from_rate).unwrap_or(1)),
        );
        Self {
            history: VecDeque::from(vec![0.0; taps.len()]),
            taps,
            factor: usize::try_from(from_rate / to_rate).unwrap_or(1).max(1),
            phase: 0,
        }
    }

    /// The 16 kHz samples of the next 48 kHz block.
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        let mut kept = Vec::with_capacity(samples.len() / self.factor + 1);
        for sample in samples {
            self.history.pop_front();
            self.history.push_back(*sample);
            self.phase += 1;
            if self.phase == self.factor {
                self.phase = 0;
                kept.push(
                    self.history
                        .iter()
                        .zip(&self.taps)
                        .map(|(value, tap)| value * tap)
                        .sum(),
                );
            }
        }
        kept
    }

    /// A Hann-windowed sinc with `length` taps and the cut-off as a fraction of the rate,
    /// scaled so a steady signal keeps its level.
    fn low_pass(length: usize, cutoff: f32) -> Vec<f32> {
        let middle = Self::float(length - 1) / 2.0;
        let taps: Vec<f32> = (0..length)
            .map(|index| {
                let position = Self::float(index) - middle;
                let sinc = if position == 0.0 {
                    2.0 * cutoff
                } else {
                    (2.0 * PI * cutoff * position).sin() / (PI * position)
                };
                let window =
                    0.5 - 0.5 * (2.0 * PI * Self::float(index) / Self::float(length - 1)).cos();
                sinc * window
            })
            .collect();
        let total: f32 = taps.iter().sum();
        taps.into_iter().map(|tap| tap / total).collect()
    }

    // Sample rates and tap indexes are far below 2^24, so an f32 holds them exactly.
    #[allow(clippy::cast_precision_loss)]
    fn float(value: usize) -> f32 {
        value as f32
    }
}
