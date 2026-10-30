use std::path::Path;

use hound::WavReader;

use crate::error::RecordingResult;

/// Reads a 16-bit WAV track back as float samples for the transcriber.
pub struct TrackReader;

impl TrackReader {
    pub fn read(path: &Path) -> RecordingResult<Vec<f32>> {
        let mut reader = WavReader::open(path)?;
        let full_scale = f32::from(i16::MAX);
        reader
            .samples::<i16>()
            .map(|sample| Ok(f32::from(sample?) / full_scale))
            .collect()
    }
}
