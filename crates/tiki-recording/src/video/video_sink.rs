use std::path::Path;

use super::{CaptureTarget, VideoPreset};
use crate::error::RecordingResult;
use crate::media::MediaSink;

/// Where a video session goes: the MP4 writer on Windows; elsewhere video cannot be recorded.
pub struct VideoSink;

impl VideoSink {
    #[cfg(windows)]
    pub fn start(
        target: &CaptureTarget,
        preset: VideoPreset,
        path: &Path,
    ) -> RecordingResult<Box<dyn MediaSink>> {
        Ok(Box::new(super::mp4_writer::Mp4Writer::start(
            target, preset, path,
        )?))
    }

    #[cfg(not(windows))]
    pub fn start(
        _target: &CaptureTarget,
        _preset: VideoPreset,
        _path: &Path,
    ) -> RecordingResult<Box<dyn MediaSink>> {
        Err(crate::error::RecordingError::UnsupportedPlatform)
    }
}
