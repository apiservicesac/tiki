use tiki_core::sessions::VideoQuality;

use crate::recording_config::RecordingConfig;

/// How a video is encoded: frames per second and bit rate. The video keeps the size of the
/// screen or window captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoPreset {
    pub frame_rate: u32,
    /// Bits per second of the H.264 video.
    pub bitrate: u32,
}

impl VideoPreset {
    pub fn for_quality(quality: VideoQuality) -> Self {
        match quality {
            VideoQuality::Normal => RecordingConfig::NORMAL_VIDEO,
            VideoQuality::High => RecordingConfig::HIGH_VIDEO,
        }
    }
}
