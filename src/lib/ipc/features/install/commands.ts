import { invoke } from "@tauri-apps/api/core";
import { Install, InstallFilter } from "./types";

export function install(
  version: string,
  flavor: string,
  variant: string,
): Promise<void> {
  return invoke<void>("installs::install", {
    version,
    flavor,
    variant,
  });
}

export function list(filter: InstallFilter): Promise<Install[]> {
  return invoke<Install[]>("installs::list", {
    filter,
  });
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

export function pause(id: string): Promise<void> {
  return invoke<void>("installs::pause", {
    id,
  });
}

export function resume(id: string): Promise<void> {
  return invoke<void>("installs::resume", {
    id,
  });
}
