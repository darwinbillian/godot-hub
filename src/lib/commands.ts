import { invoke } from "@tauri-apps/api/core";

export interface Error {
  message: string;
}

export interface Version {
  name: string;
  flavor: string;
  release_notes: string;
  status: VersionStatus;
}

export type VersionStatus =
  | { type: "available" }
  | { type: "installing" }
  | { type: "installed" }
  | { type: "failed"; error: Error };

export interface VersionUpdateEventArgs {
  name: string;
  flavor: string;
  status: VersionStatus;
}

export interface Install {
  id: string;
  dir: string;
  version: string;
  flavor: string;
}

export function show(): Promise<void> {
  return invoke<void>("show");
}

export function listVersions(): Promise<Version[]> {
  return invoke<Version[]>("list_versions");
}

export function install(version: string, flavor: string): Promise<void> {
  return invoke<void>("install", {
    version,
    flavor,
  });
}

export function listInstalls(): Promise<Install[]> {
  return invoke<Install[]>("list_installs");
}

export function launch(id: string): Promise<void> {
  return invoke<void>("launch", {
    id,
  });
}

export function uninstall(id: string): Promise<void> {
  return invoke<void>("uninstall", {
    id,
  });
}

export function reveal(id: string): Promise<void> {
  return invoke<void>("reveal", {
    id,
  });
}
