import { invoke } from "@tauri-apps/api/core";

export function show(): Promise<void> {
  return invoke<void>("show");
}
