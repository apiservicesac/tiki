use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, PoisonError};
use std::thread;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tiki_recording::RecordingResult;
use tiki_recording::transcription::{
    DownloadProgress, ModelDownloader, TranscriptionModel, TranscriptionWorker, WhisperTranscriber,
};

use crate::app_state::AppState;
use crate::core_transcription_queue::CoreTranscriptionQueue;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// How far the model download got, as the interface shows it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelDownloadStatus {
    downloaded_bytes: u64,
    total_bytes: u64,
}

/// Transcribes sessions in the background, on its own thread and one at a time: woken
/// when a recording starts (to download the model the first time) or stops, and every
/// `TRANSCRIPTION_CHECK_INTERVAL` to retry what failed.
pub struct TranscriptionRunner {
    wake: Mutex<Sender<()>>,
}

impl TranscriptionRunner {
    const PERCENT: u64 = 100;

    pub fn start(app: AppHandle, models_folder: &Path) -> std::io::Result<Self> {
        let (wake, woken) = mpsc::channel();
        let models_folder = models_folder.to_path_buf();
        thread::Builder::new()
            .name("transcription".to_owned())
            .spawn(move || Self::run(&app, &models_folder, &woken))?;
        Ok(Self {
            wake: Mutex::new(wake),
        })
    }

    /// Something changed in the queue: look at it now.
    pub fn wake(&self) {
        let _ = self
            .wake
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .send(());
    }

    fn run(app: &AppHandle, models_folder: &Path, woken: &Receiver<()>) {
        let requeued = app
            .state::<AppState>()
            .with_workspace(|workspace| workspace.sessions().requeue_interrupted_transcriptions());
        if let Err(error) = requeued {
            tracing::warn!(%error, "could not requeue the transcriptions cut short");
        }
        let downloader = match ModelDownloader::new(models_folder) {
            Ok(downloader) => downloader,
            Err(error) => {
                tracing::warn!(%error, "transcription is off: no model downloader");
                return;
            }
        };
        let mut loaded = None;
        loop {
            if let Err(error) = Self::transcribe_waiting(app, &downloader, &mut loaded) {
                tracing::warn!(%error, "transcription paused until the next try");
            }
            match woken.recv_timeout(DesktopConfig::TRANSCRIPTION_CHECK_INTERVAL) {
                Ok(()) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    /// Makes sure the chosen model is here and loaded, then empties the queue.
    fn transcribe_waiting(
        app: &AppHandle,
        downloader: &ModelDownloader,
        loaded: &mut Option<(TranscriptionModel, WhisperTranscriber)>,
    ) -> RecordingResult<()> {
        let (needed, quality) = app.state::<AppState>().with_workspace(|workspace| {
            Ok((
                workspace.sessions().awaits_transcription()?,
                workspace.preferences().transcription_quality,
            ))
        })?;
        if !needed {
            return Ok(());
        }
        let model = TranscriptionModel::for_quality(quality);
        let mut shown_percent = None;
        let path = downloader.ensure(&model, &mut |progress| {
            let percent = Self::percent(progress);
            if shown_percent != Some(percent) {
                shown_percent = Some(percent);
                ShellEvent::ModelDownloadProgress.emit_with(
                    app,
                    &ModelDownloadStatus {
                        downloaded_bytes: progress.downloaded_bytes,
                        total_bytes: progress.total_bytes,
                    },
                );
            }
        })?;
        if loaded.as_ref().is_none_or(|(current, _)| *current != model) {
            *loaded = Some((model, WhisperTranscriber::load(&path)?));
        }
        let Some((_, transcriber)) = loaded.as_mut() else {
            return Ok(());
        };
        let queue = CoreTranscriptionQueue::new(app);
        let worker = TranscriptionWorker::new(&queue);
        while worker.run_next(transcriber)? {}
        Ok(())
    }

    fn percent(progress: DownloadProgress) -> u64 {
        progress.downloaded_bytes * Self::PERCENT / progress.total_bytes.max(1)
    }
}
