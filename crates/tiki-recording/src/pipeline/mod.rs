//! The recording itself: every `PUMP_INTERVAL` the pipeline takes what the recording clock
//! says is due from both sources, writes each one's track and the mix.

mod monotonic_clock;
mod pause_clock;
mod recording_pipeline;

pub use monotonic_clock::{MonotonicClock, SystemMonotonicClock};
pub use pause_clock::PauseClock;
pub use recording_pipeline::{RecordingPipeline, RecordingTracks};
