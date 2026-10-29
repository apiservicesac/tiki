use std::time::{Duration, Instant};

/// Time since the recording started, from a clock that never jumps, so tests can drive it.
pub trait MonotonicClock: Send {
    fn elapsed(&self) -> Duration;
}

/// The computer's steady clock.
pub struct SystemMonotonicClock {
    started: Instant,
}

impl Default for SystemMonotonicClock {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}
