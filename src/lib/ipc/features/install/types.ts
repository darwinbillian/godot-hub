import { Error } from "@/lib/ipc/types";

export interface Install {
  id: string;
  version: string;
  flavor: string;
  status: InstallStatus;
}

export type InstallStatus =
  | { type: "installing" }
  | { type: "installed"; installation: Installation }
  | { type: "failed"; error: Error };

export interface Installation {
  dir: string;
}

export interface InstallUpdateEventArgs {
  id: string;
  status: InstallStatus;
}

export interface InstallRemoveEventArgs {
  id: string;
}
