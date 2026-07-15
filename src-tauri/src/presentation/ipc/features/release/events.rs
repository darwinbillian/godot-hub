use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::ReleaseUpdateEventArgsDto;
use crate::application::{services::release::ReleaseUpdateEventArgs, utils::event::EventHandler};

pub struct ReleaseUpdateEventEmitter {
    app: AppHandle,
}

impl ReleaseUpdateEventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventHandler<ReleaseUpdateEventArgs> for ReleaseUpdateEventEmitter {
    fn invoke(&self, args: Arc<ReleaseUpdateEventArgs>) {
        let _ = self
            .app
            .emit("releases::update", &ReleaseUpdateEventArgsDto::from(args));
    }
}
