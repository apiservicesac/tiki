use super::AudioSource;
use crate::error::RecordingResult;

/// The computer's default microphone and its system audio (what plays on the speakers or
/// headphones: the other people in a call). Only Windows can capture them for now.
pub struct CaptureDevices;

impl CaptureDevices {
    #[cfg(windows)]
    pub fn microphone() -> RecordingResult<Box<dyn AudioSource>> {
        Ok(Box::new(super::wasapi_source::WasapiSource::start(
            super::wasapi_source::WasapiEndpoint::Microphone,
        )?))
    }

    #[cfg(windows)]
    pub fn system_audio() -> RecordingResult<Box<dyn AudioSource>> {
        Ok(Box::new(super::wasapi_source::WasapiSource::start(
            super::wasapi_source::WasapiEndpoint::SystemAudio,
        )?))
    }

    #[cfg(not(windows))]
    pub fn microphone() -> RecordingResult<Box<dyn AudioSource>> {
        Err(crate::error::RecordingError::UnsupportedPlatform)
    }

    #[cfg(not(windows))]
    pub fn system_audio() -> RecordingResult<Box<dyn AudioSource>> {
        Err(crate::error::RecordingError::UnsupportedPlatform)
    }
}
