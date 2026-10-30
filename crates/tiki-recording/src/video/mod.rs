//! Video sessions: what can be captured (a screen or one window), the encoding presets and,
//! on Windows, the MP4 writer fed by Windows.Graphics.Capture.

mod capture_target;
#[cfg(windows)]
mod mp4_writer;
mod video_preset;
mod video_sink;

pub use capture_target::{CaptureKind, CaptureTarget};
pub use video_preset::VideoPreset;
pub use video_sink::VideoSink;
