pub use tokio_util::sync::CancellationToken;

pub trait CancellationTokenExt {
    fn error_if_cancelled(&self) -> Result<(), CancellationError>;
}

pub struct CancellationError;

impl CancellationError {
    pub fn new() -> Self {
        Self
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
