import { invoke } from "@tauri-apps/api/core";
import { Version } from "./types";

export function list(): Promise<Version[]> {
  return invoke<Version[]>("versions::list");
}
