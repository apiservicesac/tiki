use serde::{Deserialize, Serialize};
use tiki_core::sessions::VideoQuality;
use tiki_recording::video::{CaptureKind, CaptureTarget};

/// A screen or a window, as the interface lists and picks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureTargetKind {
    Screen,
    Window,
}

/// A screen or a window the user can record.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTargetView {
    kind: CaptureTargetKind,
    id: usize,
    /// A window's title; screens are named by their number in the interface.
    name: String,
}

impl From<CaptureTarget> for CaptureTargetView {
    fn from(target: CaptureTarget) -> Self {
        Self {
            kind: match target.kind {
                CaptureKind::Screen => CaptureTargetKind::Screen,
                CaptureKind::Window => CaptureTargetKind::Window,
            },
            id: target.id,
            name: target.name,
        }
    }
}

/// What a video session records and how, as the user picked it.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoChoice {
    pub kind: CaptureTargetKind,
    pub id: usize,
    pub quality: VideoQuality,
}

impl VideoChoice {
    pub fn target(self) -> CaptureTarget {
        CaptureTarget {
            kind: match self.kind {
                CaptureTargetKind::Screen => CaptureKind::Screen,
                CaptureTargetKind::Window => CaptureKind::Window,
            },
            id: self.id,
            name: String::new(),
        }
    }
}
