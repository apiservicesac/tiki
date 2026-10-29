/// Converts float samples (-1.0 to 1.0) to the 16-bit integers WAV tracks and encoders take.
pub struct SampleFormat;

impl SampleFormat {
    // The sample is clamped and scaled to the i16 range first, so the cast cannot overflow.
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_i16(sample: f32) -> i16 {
        (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16
    }

    /// Little-endian 16-bit PCM bytes, as Windows' encoders read them.
    pub fn to_pcm16_bytes(samples: &[f32]) -> Vec<u8> {
        samples
            .iter()
            .flat_map(|sample| Self::to_i16(*sample).to_le_bytes())
            .collect()
    }
}
