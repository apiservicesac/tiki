use std::time::Duration;

/// The recorded time on the media timeline: what has passed since the start, minus every
/// pause. Nothing said during a pause reaches the files, and the timeline does not jump.
#[derive(Debug, Default)]
pub struct PauseClock {
    paused_before: Duration,
    paused_since: Option<Duration>,
}

impl PauseClock {
    pub fn pause(&mut self, elapsed: Duration) {
        self.paused_since.get_or_insert(elapsed);
    }

    pub fn resume(&mut self, elapsed: Duration) {
        if let Some(since) = self.paused_since.take() {
            self.paused_before += elapsed.saturating_sub(since);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_since.is_some()
    }

    /// Recorded time when `elapsed` has passed since the start.
    pub fn recorded(&self, elapsed: Duration) -> Duration {
        let pause_in_progress = self
            .paused_since
            .map_or(Duration::ZERO, |since| elapsed.saturating_sub(since));
        elapsed
            .saturating_sub(self.paused_before)
            .saturating_sub(pause_in_progress)
    }
}
