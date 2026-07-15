import { invoke } from "@tauri-apps/api/core";
import { Release } from "./types";

export function list(): Promise<Release[]> {
  return invoke<Release[]>("releases::list");
}
