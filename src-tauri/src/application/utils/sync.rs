pub use tokio_util::sync::CancellationToken;

use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Notify;

pub trait CancellationTokenExt {
    fn error_if_cancelled(&self) -> Result<(), CancellationError>;
}

pub struct CancellationError;

pub struct PauseToken {
    notify: Notify,
    paused: AtomicBool,
}

impl CancellationError {
    pub fn new() -> Self {
        Self
    }
}

impl PauseToken {
    pub fn new() -> Self {
        Self {
            notify: Notify::new(),
            paused: AtomicBool::new(false),
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Release);
    }

    pub fn resume(&self) {
        if self.paused.swap(false, Ordering::AcqRel) {
            self.notify.notify_waiters();
        }
    }

    pub async fn paused(&self) {
        let notified = self.notify.notified();
        if self.is_paused() {
            notified.await;
        }
    }
}

impl CancellationTokenExt for CancellationToken {
    fn error_if_cancelled(&self) -> Result<(), CancellationError> {
        if self.is_cancelled() {
            Err(CancellationError::new())
        } else {
            Ok(())
        }
    }
}
