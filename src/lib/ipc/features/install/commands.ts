import { invoke } from "@tauri-apps/api/core";
import { Install } from "./types";

export function install(version: string, flavor: string): Promise<void> {
  return invoke<void>("installs::install", {
    version,
    flavor,
  });
}

export function list(): Promise<Install[]> {
  return invoke<Install[]>("installs::list");
}

export function launch(id: string): Promise<void> {
  return invoke<void>("installs::launch", {
    id,
  });
}

export function uninstall(id: string): Promise<void> {
  return invoke<void>("installs::uninstall", {
    id,
  });
}

export function reveal(id: string): Promise<void> {
  return invoke<void>("installs::reveal", {
    id,
  });
}

export function cancel(id: string): Promise<void> {
  return invoke<void>("installs::cancel", {
    id,
  });
}
