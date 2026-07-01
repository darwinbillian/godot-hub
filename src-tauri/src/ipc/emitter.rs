use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::dtos::{
    InstallRemoveEventArgsDto, InstallUpdateEventArgsDto, VersionUpdateEventArgsDto,
};
use crate::{
    event::EventHandler,
    services::{
        install::{InstallRemoveEventArgs, InstallUpdateEventArgs},
        version::VersionUpdateEventArgs,
    },
};

pub struct VersionUpdateEmitter {
    app: AppHandle,
}

pub struct InstallUpdateEmitter {
    app: AppHandle,
}

pub struct InstallRemoveEmitter {
    app: AppHandle,
}

impl VersionUpdateEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
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

impl EventHandler<VersionUpdateEventArgs> for VersionUpdateEmitter {
    fn invoke(&self, args: Arc<VersionUpdateEventArgs>) {
        let _ = self
            .app
            .emit("update_version", &VersionUpdateEventArgsDto::from(args));
    }
}

impl EventHandler<InstallUpdateEventArgs> for InstallUpdateEmitter {
    fn invoke(&self, args: Arc<InstallUpdateEventArgs>) {
        let _ = self
            .app
            .emit("update_install", &InstallUpdateEventArgsDto::from(args));
    }
}

impl EventHandler<InstallRemoveEventArgs> for InstallRemoveEmitter {
    fn invoke(&self, args: Arc<InstallRemoveEventArgs>) {
        let _ = self
            .app
            .emit("remove_install", &InstallRemoveEventArgsDto::from(args));
    }
}
