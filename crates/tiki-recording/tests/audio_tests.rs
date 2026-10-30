use std::f32::consts::PI;

use tiki_recording::RecordingConfig;
use tiki_recording::audio::{AudioMixer, Decimator, SampleFormat};

const RATE: f32 = 48_000.0;

fn tone(frequency: f32, samples: usize) -> Vec<f32> {
    (0..samples)
        .map(|index| {
            (2.0 * PI * frequency * f32::from(u16::try_from(index % 48_000).unwrap()) / RATE).sin()
        })
        .collect()
}

fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0, |peak, sample| peak.max(sample.abs()))
}

#[test]
fn the_mix_adds_both_sources_without_clipping_past_full_scale() {
    let mixed = AudioMixer::default().mix(&[0.25, 0.8, -0.9], &[0.25, 0.7, -0.6]);
    assert_eq!(mixed, [0.5, 1.0, -1.0]);
}

#[test]
fn decimation_keeps_one_sample_in_three_across_blocks() {
    let mut decimator = Decimator::default();
    let first = decimator.process(&vec![0.0; 1_000]);
    let second = decimator.process(&vec![0.0; 2_000]);
    assert_eq!(first.len() + second.len(), 1_000);
    let ratio = RecordingConfig::MIX_SAMPLE_RATE / RecordingConfig::TRANSCRIPTION_SAMPLE_RATE;
    assert_eq!(ratio, 3);
}

#[test]
fn decimation_keeps_speech_and_removes_what_16_khz_cannot_hold() {
    let samples = 48_000;
    let speech = Decimator::default().process(&tone(440.0, samples));
    let hiss = Decimator::default().process(&tone(12_000.0, samples));
    // The filter needs its length to settle; measure after it.
    let settled = RecordingConfig::DECIMATION_TAPS;
    assert!(peak(&speech[settled..]) > 0.95);
    assert!(peak(&hiss[settled..]) < 0.05);
}

#[test]
fn samples_become_16_bit_integers_within_range() {
    assert_eq!(SampleFormat::to_i16(1.5), i16::MAX);
    assert_eq!(SampleFormat::to_i16(-1.0), -i16::MAX);
    assert_eq!(
        SampleFormat::to_pcm16_bytes(&[0.0, 1.0]),
        [0, 0, 0xFF, 0x7F]
    );
}
