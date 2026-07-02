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
