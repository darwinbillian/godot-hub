use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::{InstallRemoveEventArgsDto, InstallUpdateEventArgsDto};
use crate::{
    application::services::install::{InstallRemoveEventArgs, InstallUpdateEventArgs},
    event::EventHandler,
};

pub struct InstallUpdateEventEmitter {
    app: AppHandle,
}

pub struct InstallRemoveEventEmitter {
    app: AppHandle,
}

impl InstallUpdateEventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl InstallRemoveEventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventHandler<InstallUpdateEventArgs> for InstallUpdateEventEmitter {
    fn invoke(&self, args: Arc<InstallUpdateEventArgs>) {
        let _ = self
            .app
            .emit("installs::update", &InstallUpdateEventArgsDto::from(args));
    }
}

impl EventHandler<InstallRemoveEventArgs> for InstallRemoveEventEmitter {
    fn invoke(&self, args: Arc<InstallRemoveEventArgs>) {
        let _ = self
            .app
            .emit("installs::remove", &InstallRemoveEventArgsDto::from(args));
    }
}
