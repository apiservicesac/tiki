use std::collections::VecDeque;

use super::AudioSource;
use crate::error::RecordingResult;

/// Keeps one source in step with the recording clock. The clock says how many samples are
/// due; a source that delivered fewer (silence, a late device) is filled with silence, and
/// one that runs ahead (its device ticks a little faster) loses its oldest surplus beyond
/// `max_backlog`, so both sources stay aligned for hours.
pub struct SourceAligner {
    backlog: VecDeque<f32>,
    max_backlog: usize,
    incoming: Vec<f32>,
}

impl SourceAligner {
    pub fn new(max_backlog: usize) -> Self {
        Self {
            backlog: VecDeque::new(),
            max_backlog,
            incoming: Vec::new(),
        }
    }

    /// Takes in what the source captured since the last call.
    pub fn receive(&mut self, source: &mut dyn AudioSource) -> RecordingResult<()> {
        source.drain_into(&mut self.incoming)?;
        self.backlog.extend(self.incoming.drain(..));
        Ok(())
    }

    /// Throws away what the source captured, such as everything said during a pause.
    pub fn discard(&mut self, source: &mut dyn AudioSource) -> RecordingResult<()> {
        self.receive(source)?;
        self.backlog.clear();
        Ok(())
    }

    /// The next `due` samples: silence where the source fell short.
    pub fn take(&mut self, due: usize) -> Vec<f32> {
        let available = due.min(self.backlog.len());
        let mut samples: Vec<f32> = self.backlog.drain(..available).collect();
        samples.resize(due, 0.0);
        let surplus = self.backlog.len().saturating_sub(self.max_backlog);
        self.backlog.drain(..surplus);
        samples
    }
}
