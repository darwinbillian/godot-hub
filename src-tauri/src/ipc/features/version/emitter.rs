use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::VersionUpdateEventArgsDto;
use crate::{event::EventHandler, services::version::VersionUpdateEventArgs};

pub struct VersionUpdateEmitter {
    app: AppHandle,
}

impl VersionUpdateEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventHandler<VersionUpdateEventArgs> for VersionUpdateEmitter {
    fn invoke(&self, args: Arc<VersionUpdateEventArgs>) {
        let _ = self
            .app
            .emit("versions::update", &VersionUpdateEventArgsDto::from(args));
    }
}
