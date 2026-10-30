use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The one sign-in waiting for the browser. Starting another one or cancelling stops it,
/// so a closed browser window never leaves Tiki waiting.
#[derive(Default)]
pub struct PendingSignIn {
    cancelled: Mutex<Option<Arc<AtomicBool>>>,
}

impl PendingSignIn {
    /// Stops the sign-in waiting, if any, and gives the flag that stops the new one.
    pub fn begin(&self) -> Arc<AtomicBool> {
        let cancelled = Arc::new(AtomicBool::new(false));
        if let Some(previous) = self.current().replace(Arc::clone(&cancelled)) {
            previous.store(true, Ordering::Relaxed);
        }
        cancelled
    }

    /// Stops the sign-in waiting, if any.
    pub fn cancel(&self) {
        if let Some(previous) = self.current().take() {
            previous.store(true, Ordering::Relaxed);
        }
    }

    fn current(&self) -> MutexGuard<'_, Option<Arc<AtomicBool>>> {
        self.cancelled
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
