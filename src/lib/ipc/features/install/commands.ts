import { invoke } from "@tauri-apps/api/core";
import { Install } from "./types";

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
