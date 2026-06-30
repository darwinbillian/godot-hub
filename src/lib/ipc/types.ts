export interface Error {
  message: string;
}

export interface Version {
  name: string;
  flavor: string;
  release_notes: string;
  status: VersionStatus;
}

export type VersionStatus =
  | { type: "available" }
  | { type: "installing" }
  | { type: "installed" }
  | { type: "failed"; error: Error };

export interface VersionUpdateEventArgs {
  name: string;
  flavor: string;
  status: VersionStatus;
}

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
