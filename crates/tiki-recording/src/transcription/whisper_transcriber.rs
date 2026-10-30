use std::path::Path;
use std::thread;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperError,
};

use super::{TimedText, TranscribedAudio, Transcriber};
use crate::error::{RecordingError, RecordingResult};
use crate::recording_config::RecordingConfig;

/// Whisper (whisper.cpp) on the processor: nothing leaves the computer. It detects the
/// language and drops segments that are most likely silence.
pub struct WhisperTranscriber {
    context: WhisperContext,
    threads: i32,
}

impl WhisperTranscriber {
    pub fn load(model: &Path) -> RecordingResult<Self> {
        whisper_rs::install_logging_hooks();
        let context = WhisperContext::new_with_params(model, WhisperContextParameters::default())
            .map_err(Self::failure)?;
        let cores = thread::available_parallelism().map_or(1, std::num::NonZero::get);
        let threads = (cores / RecordingConfig::TRANSCRIPTION_CORE_DIVISOR).max(1);
        Ok(Self {
            context,
            threads: i32::try_from(threads).unwrap_or(1),
        })
    }

    fn failure(error: WhisperError) -> RecordingError {
        RecordingError::Transcription(error.to_string())
    }
}

impl Transcriber for WhisperTranscriber {
    fn transcribe(&mut self, samples: &[f32]) -> RecordingResult<TranscribedAudio> {
        let mut state = self.context.create_state().map_err(Self::failure)?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(RecordingConfig::WHISPER_AUTO_LANGUAGE));
        params.set_n_threads(self.threads);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        state.full(params, samples).map_err(Self::failure)?;
        let language = whisper_rs::get_lang_str(state.full_lang_id_from_state()).map(str::to_owned);
        let mut segments = Vec::new();
        for segment in state.as_iter() {
            if segment.no_speech_probability() > RecordingConfig::NO_SPEECH_PROBABILITY_LIMIT {
                continue;
            }
            segments.push(TimedText {
                start_ms: segment.start_timestamp() * RecordingConfig::WHISPER_TIMESTAMP_MS,
                end_ms: segment.end_timestamp() * RecordingConfig::WHISPER_TIMESTAMP_MS,
                text: segment.to_str().map_err(Self::failure)?.trim().to_owned(),
            });
        }
        Ok(TranscribedAudio { language, segments })
    }
}
