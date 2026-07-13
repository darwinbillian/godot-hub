use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::VersionUpdateEventArgsDto;
use crate::application::{services::version::VersionUpdateEventArgs, utils::event::EventHandler};

pub struct VersionUpdateEventEmitter {
    app: AppHandle,
}

impl VersionUpdateEventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventHandler<VersionUpdateEventArgs> for VersionUpdateEventEmitter {
    fn invoke(&self, args: Arc<VersionUpdateEventArgs>) {
        let _ = self
            .app
            .emit("versions::update", &VersionUpdateEventArgsDto::from(args));
    }
}
