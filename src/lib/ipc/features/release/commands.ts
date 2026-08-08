import { invoke } from "@tauri-apps/api/core";
import { Release, ReleaseFilter } from "./types";

export function list(filter: ReleaseFilter): Promise<Release[]> {
  return invoke<Release[]>("releases::list", {
    filter,
  });
}
