import { Error } from "@/lib/ipc/types";

export interface Install {
  id: string;
  version: string;
  flavor: string;
  status: InstallStatus;
}

export type InstallStatus =
  | { type: "installing"; progress: InstallProgress }
  | { type: "installed"; installation: Installation }
  | { type: "failed"; error: Error };

export type InstallProgress =
  | { type: "starting" }
  | { type: "downloading" }
  | { type: "extracting" }
  | { type: "finalizing" };

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
