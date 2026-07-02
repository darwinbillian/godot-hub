import { invoke } from "@tauri-apps/api/core";
import { Version } from "./types";

export function listVersions(): Promise<Version[]> {
  return invoke<Version[]>("list_versions");
}
