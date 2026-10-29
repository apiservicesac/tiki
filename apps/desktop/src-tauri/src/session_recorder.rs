use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;

use tauri::{AppHandle, Manager};
use tiki_core::sessions::{RecordingMode, Session, SessionId};
use tiki_recording::audio::CaptureDevices;
use tiki_recording::media::{OggOpusWriter, TrackWriter};
use tiki_recording::pipeline::{RecordingPipeline, RecordingTracks, SystemMonotonicClock};
use tiki_recording::{RecordingConfig, RecordingResult};

use crate::app_state::AppState;
use crate::recorder_error::RecorderError;
use crate::session_files::SessionFiles;
use crate::shell_events::ShellEvent;
use crate::transcription_runner::TranscriptionRunner;

/// What the recorder thread is asked to do.
enum RecorderCommand {
    Pause,
    Resume,
    Stop,
}

/// The session being recorded and the thread recording it.
struct RunningRecorder {
    session_id: SessionId,
    commands: Sender<RecorderCommand>,
}

/// Records the session the user started: the core keeps the session and its rules, a
/// thread pumps the microphone and the system audio into the session's folder. Stopping
/// hands the finished media to the core, which queues the transcription.
#[derive(Default)]
pub struct SessionRecorder {
    running: Mutex<Option<RunningRecorder>>,
}

impl SessionRecorder {
    pub fn start(&self, app: &AppHandle, mode: RecordingMode) -> Result<(), RecorderError> {
        let mut running = self.lock();
        if mode == RecordingMode::AudioAndVideo {
            return Err(RecorderError::VideoUnavailable);
        }
        let state = app.state::<AppState>();
        let session = state.with_workspace(|workspace| workspace.sessions().start(mode))?;
        let session_id = session.id.ok_or(tiki_core::TikiError::UnsavedSession)?;
        let pipeline = match Self::open_pipeline(&app.state::<SessionFiles>(), &session) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                if let Err(abandon_error) =
                    state.with_workspace(|workspace| workspace.sessions().abandon(session_id))
                {
                    tracing::warn!(%abandon_error, "could not forget the session that did not start");
                }
                return Err(error.into());
            }
        };
        let (commands, received) = mpsc::channel();
        let recorder_app = app.clone();
        thread::Builder::new()
            .name("session recorder".to_owned())
            .spawn(move || Self::record(&recorder_app, session_id, pipeline, &received))?;
        *running = Some(RunningRecorder {
            session_id,
            commands,
        });
        tracing::info!(?session_id, ?mode, "recording started");
        ShellEvent::RecordingChanged.emit(app);
        // The first recording downloads the transcription model meanwhile.
        app.state::<TranscriptionRunner>().wake();
        Ok(())
    }

    pub fn pause(&self, app: &AppHandle) -> Result<(), RecorderError> {
        self.command(app, RecorderCommand::Pause, |session_id| {
            app.state::<AppState>()
                .with_workspace(|workspace| workspace.sessions().pause(session_id).map(drop))
        })
    }

    pub fn resume(&self, app: &AppHandle) -> Result<(), RecorderError> {
        self.command(app, RecorderCommand::Resume, |session_id| {
            app.state::<AppState>()
                .with_workspace(|workspace| workspace.sessions().resume(session_id).map(drop))
        })
    }

    /// Asks the thread to finish the files; it hands them to the core when done.
    pub fn stop(&self) -> Result<(), RecorderError> {
        let running = self.lock().take().ok_or(RecorderError::NotRecording)?;
        running
            .commands
            .send(RecorderCommand::Stop)
            .map_err(|_| RecorderError::NotRecording)
    }

    /// Changes the session in the core, then tells the thread.
    fn command(
        &self,
        app: &AppHandle,
        command: RecorderCommand,
        change: impl FnOnce(SessionId) -> tiki_core::TikiResult<()>,
    ) -> Result<(), RecorderError> {
        let running = self.lock();
        let running = running.as_ref().ok_or(RecorderError::NotRecording)?;
        change(running.session_id)?;
        running
            .commands
            .send(command)
            .map_err(|_| RecorderError::NotRecording)?;
        ShellEvent::RecordingChanged.emit(app);
        Ok(())
    }

    fn open_pipeline(files: &SessionFiles, session: &Session) -> RecordingResult<RecordingPipeline> {
        let folder = files.prepare(session)?;
        let tracks = RecordingTracks {
            microphone: CaptureDevices::microphone()?,
            system: CaptureDevices::system_audio()?,
            microphone_track: TrackWriter::create(&folder.join(RecordingConfig::MICROPHONE_TRACK_FILE))?,
            system_track: TrackWriter::create(&folder.join(RecordingConfig::SYSTEM_TRACK_FILE))?,
            media: Box::new(OggOpusWriter::create(&folder.join(RecordingConfig::AUDIO_FILE))?),
        };
        Ok(RecordingPipeline::new(
            tracks,
            Box::new(SystemMonotonicClock::default()),
        ))
    }

    /// The recorder thread: pumps until stopped (or a device fails), then closes the files
    /// and stops the session in the core.
    fn record(
        app: &AppHandle,
        session_id: SessionId,
        mut pipeline: RecordingPipeline,
        commands: &Receiver<RecorderCommand>,
    ) {
        if let Err(error) = Self::pump_until_stopped(&mut pipeline, commands) {
            tracing::warn!(%error, "the recording stopped early");
        }
        let media_bytes = pipeline.finish().unwrap_or_else(|error| {
            tracing::warn!(%error, "could not finish the session's media");
            0
        });
        let stopped = app.state::<AppState>().with_workspace(|workspace| {
            workspace
                .sessions()
                .stop(session_id, RecordingConfig::AUDIO_FILE.to_owned(), media_bytes)
                .map(drop)
        });
        match stopped {
            Ok(()) => tracing::info!(?session_id, media_bytes, "recording stopped"),
            Err(error) => tracing::warn!(%error, "could not stop the session in the core"),
        }
        app.state::<SessionRecorder>().forget(session_id);
        ShellEvent::RecordingChanged.emit(app);
        app.state::<TranscriptionRunner>().wake();
    }

    fn pump_until_stopped(
        pipeline: &mut RecordingPipeline,
        commands: &Receiver<RecorderCommand>,
    ) -> RecordingResult<()> {
        loop {
            match commands.recv_timeout(RecordingConfig::PUMP_INTERVAL) {
                Ok(RecorderCommand::Pause) => pipeline.pause()?,
                Ok(RecorderCommand::Resume) => pipeline.resume()?,
                Ok(RecorderCommand::Stop) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
                Err(RecvTimeoutError::Timeout) => pipeline.pump()?,
            }
        }
    }

    /// The thread of `session_id` ended by itself (a device failed): it no longer runs.
    fn forget(&self, session_id: SessionId) {
        let mut running = self.lock();
        if running
            .as_ref()
            .is_some_and(|recorder| recorder.session_id == session_id)
        {
            *running = None;
        }
    }

    fn lock(&self) -> MutexGuard<'_, Option<RunningRecorder>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
