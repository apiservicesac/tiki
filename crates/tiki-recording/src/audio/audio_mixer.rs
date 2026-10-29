use crate::recording_config::RecordingConfig;

/// Mixes the microphone and the system audio into the one track played back, each with
/// its gain, clipped to the valid range.
pub struct AudioMixer {
    microphone_gain: f32,
    system_gain: f32,
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self {
            microphone_gain: RecordingConfig::MICROPHONE_GAIN,
            system_gain: RecordingConfig::SYSTEM_GAIN,
        }
    }
}

impl AudioMixer {
    /// Both slices are as long as each other: the aligners took the same samples due.
    pub fn mix(&self, microphone: &[f32], system: &[f32]) -> Vec<f32> {
        microphone
            .iter()
            .zip(system)
            .map(|(spoken, heard)| {
                (spoken * self.microphone_gain + heard * self.system_gain).clamp(-1.0, 1.0)
            })
            .collect()
    }
}
