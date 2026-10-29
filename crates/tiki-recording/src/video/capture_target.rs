use crate::error::RecordingResult;

/// What a video session captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    /// A whole screen; its id is its number, from 1.
    Screen,
    /// One window; its id is its handle.
    Window,
}

/// A screen or a window the user can record, with the name the picker shows (a window's
/// title; screens are named by their number in the interface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTarget {
    pub kind: CaptureKind,
    pub id: usize,
    pub name: String,
}

impl CaptureTarget {
    /// Every screen, then every window the user could see, Tiki's own left out.
    #[cfg(windows)]
    pub fn list() -> RecordingResult<Vec<Self>> {
        use windows_capture::monitor::Monitor;
        use windows_capture::window::Window;

        let capture_failed = |reason: String| crate::error::RecordingError::Capture {
            source_name: "screen",
            reason,
        };
        let mut targets = Vec::new();
        for monitor in Monitor::enumerate().map_err(|error| capture_failed(error.to_string()))? {
            targets.push(Self {
                kind: CaptureKind::Screen,
                id: monitor
                    .index()
                    .map_err(|error| capture_failed(error.to_string()))?,
                name: String::new(),
            });
        }
        let own_process = std::process::id();
        for window in Window::enumerate().map_err(|error| capture_failed(error.to_string()))? {
            let title = window.title().unwrap_or_default();
            if title.trim().is_empty()
                || window
                    .process_id()
                    .is_ok_and(|process| process == own_process)
            {
                continue;
            }
            targets.push(Self {
                kind: CaptureKind::Window,
                id: window.as_raw_hwnd().addr(),
                name: title,
            });
        }
        Ok(targets)
    }

    /// Video capture needs Windows: elsewhere there is nothing to pick.
    #[cfg(not(windows))]
    pub fn list() -> RecordingResult<Vec<Self>> {
        Ok(Vec::new())
    }
}
