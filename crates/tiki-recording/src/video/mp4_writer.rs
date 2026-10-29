use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use windows_capture::capture::{
    CaptureControl, Context, GraphicsCaptureApiError, GraphicsCaptureApiHandler,
};
use windows_capture::encoder::{
    AudioSettingsBuilder, AudioSettingsSubType, ContainerSettingsBuilder, VideoEncoder,
    VideoEncoderError, VideoSettingsBuilder, VideoSettingsSubType,
};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    GraphicsCaptureItemType, MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

use super::{CaptureKind, CaptureTarget, VideoPreset};
use crate::audio::SampleFormat;
use crate::error::{RecordingError, RecordingResult};
use crate::media::MediaSink;
use crate::recording_config::RecordingConfig;

/// What the capture thread needs to open the encoder.
struct EncoderSettings {
    width: u32,
    height: u32,
    preset: VideoPreset,
    path: PathBuf,
}

/// Receives the captured frames on Windows' capture thread and hands them to the encoder.
struct FrameCapture {
    encoder: Option<VideoEncoder>,
}

impl GraphicsCaptureApiHandler for FrameCapture {
    type Flags = EncoderSettings;
    type Error = VideoEncoderError;

    fn new(context: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let settings = context.flags;
        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(settings.width, settings.height)
                .sub_type(VideoSettingsSubType::H264)
                .bitrate(settings.preset.bitrate)
                .frame_rate(settings.preset.frame_rate),
            AudioSettingsBuilder::new()
                .sub_type(AudioSettingsSubType::AAC)
                .bitrate(RecordingConfig::VIDEO_AUDIO_BITRATE)
                .sample_rate(RecordingConfig::MIX_SAMPLE_RATE)
                .channel_count(RecordingConfig::VIDEO_AUDIO_CHANNELS)
                .bit_per_sample(RecordingConfig::VIDEO_AUDIO_BITS),
            ContainerSettingsBuilder::new(),
            &settings.path,
        )?;
        Ok(Self {
            encoder: Some(encoder),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        match self.encoder.as_mut() {
            Some(encoder) => encoder.send_frame(frame),
            None => Ok(()),
        }
    }
}

/// A video session as MP4 (H.264 and AAC) through Windows' encoder: Windows.Graphics.Capture
/// sends the frames of a screen or a window, and the recorder sends the audio mix. The video
/// keeps the size of what it captures.
pub struct Mp4Writer {
    path: PathBuf,
    capture: Option<CaptureControl<FrameCapture, VideoEncoderError>>,
}

impl Mp4Writer {
    /// Starts capturing `target` into the MP4 at `path`.
    pub fn start(
        target: &CaptureTarget,
        preset: VideoPreset,
        path: &Path,
    ) -> RecordingResult<Self> {
        let capture = Self::open(target, preset, path).map_err(|error| Self::failure(&error))?;
        Ok(Self {
            path: path.to_path_buf(),
            capture: Some(capture),
        })
    }

    fn open(
        target: &CaptureTarget,
        preset: VideoPreset,
        path: &Path,
    ) -> Result<CaptureControl<FrameCapture, VideoEncoderError>, Box<dyn Error>> {
        Ok(match target.kind {
            CaptureKind::Screen => {
                let monitor = Monitor::from_index(target.id)?;
                let (width, height) = (monitor.width()?, monitor.height()?);
                Self::capture(monitor, width, height, preset, path)?
            }
            CaptureKind::Window => {
                let window = Window::from_raw_hwnd(std::ptr::without_provenance_mut(target.id));
                let width = u32::try_from(window.width()?)?;
                let height = u32::try_from(window.height()?)?;
                Self::capture(window, width, height, preset, path)?
            }
        })
    }

    fn capture<Item: TryInto<GraphicsCaptureItemType> + Send + 'static>(
        item: Item,
        width: u32,
        height: u32,
        preset: VideoPreset,
        path: &Path,
    ) -> Result<
        CaptureControl<FrameCapture, VideoEncoderError>,
        GraphicsCaptureApiError<VideoEncoderError>,
    > {
        let settings = Settings::new(
            item,
            CursorCaptureSettings::Default,
            DrawBorderSettings::Default,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Rgba8,
            EncoderSettings {
                width: Self::even(width),
                height: Self::even(height),
                preset,
                path: path.to_path_buf(),
            },
        );
        FrameCapture::start_free_threaded(settings)
    }

    /// H.264 needs even sizes.
    fn even(pixels: u32) -> u32 {
        pixels - pixels % 2
    }

    fn failure(error: &impl Display) -> RecordingError {
        RecordingError::Encoder(error.to_string())
    }
}

impl MediaSink for Mp4Writer {
    fn write_audio(&mut self, mixed: &[f32]) -> RecordingResult<()> {
        let Some(capture) = self.capture.as_ref() else {
            return Ok(());
        };
        let callback = capture.callback();
        let mut frames = callback.lock();
        match frames.encoder.as_mut() {
            Some(encoder) => encoder
                .send_audio_buffer(&SampleFormat::to_pcm16_bytes(mixed), 0)
                .map_err(|error| Self::failure(&error)),
            None => Ok(()),
        }
    }

    fn finish(mut self: Box<Self>) -> RecordingResult<u64> {
        if let Some(capture) = self.capture.take() {
            let encoder = capture.callback().lock().encoder.take();
            capture.stop().map_err(|error| Self::failure(&error))?;
            if let Some(encoder) = encoder {
                encoder.finish().map_err(|error| Self::failure(&error))?;
            }
        }
        Ok(std::fs::metadata(&self.path)?.len())
    }
}

impl Drop for Mp4Writer {
    fn drop(&mut self) {
        if let Some(capture) = self.capture.take() {
            let _ = capture.stop();
        }
    }
}
