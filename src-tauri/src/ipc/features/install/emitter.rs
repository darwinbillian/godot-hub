use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::{InstallRemoveEventArgsDto, InstallUpdateEventArgsDto};
use crate::{
    event::EventHandler,
    services::install::{InstallRemoveEventArgs, InstallUpdateEventArgs},
};

pub struct InstallUpdateEmitter {
    app: AppHandle,
}

pub struct InstallRemoveEmitter {
    app: AppHandle,
}

impl InstallUpdateEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl InstallRemoveEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventHandler<InstallUpdateEventArgs> for InstallUpdateEmitter {
    fn invoke(&self, args: Arc<InstallUpdateEventArgs>) {
        let _ = self
            .app
            .emit("installs::update", &InstallUpdateEventArgsDto::from(args));
    }
}

impl EventHandler<InstallRemoveEventArgs> for InstallRemoveEmitter {
    fn invoke(&self, args: Arc<InstallRemoveEventArgs>) {
        let _ = self
            .app
            .emit("installs::remove", &InstallRemoveEventArgsDto::from(args));
    }
}
