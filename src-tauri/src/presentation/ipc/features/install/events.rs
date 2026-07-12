use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::{InstallAddEventArgsDto, InstallRemoveEventArgsDto, InstallUpdateEventArgsDto};
use crate::application::{
    event::EventHandler,
    services::install::{InstallAddEventArgs, InstallRemoveEventArgs, InstallUpdateEventArgs},
};

pub struct InstallAddEventEmitter {
    app: AppHandle,
}

pub struct InstallUpdateEventEmitter {
    app: AppHandle,
}

pub struct InstallRemoveEventEmitter {
    app: AppHandle,
}

impl InstallAddEventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
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

impl EventHandler<InstallAddEventArgs> for InstallAddEventEmitter {
    fn invoke(&self, args: Arc<InstallAddEventArgs>) {
        let _ = self
            .app
            .emit("installs::add", &InstallAddEventArgsDto::from(args));
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
