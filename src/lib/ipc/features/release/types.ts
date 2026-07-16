export interface Release {
  name: string;
  flavor: string;
  release_notes: string;
  status: ReleaseStatus;
}

export type ReleaseStatus =
  | { type: "available" }
  | { type: "paused" }
  | { type: "installing" }
  | { type: "installed" }
  | { type: "failed"; error: Error };

export interface ReleaseUpdateEventArgs {
  name: string;
  flavor: string;
  status: ReleaseStatus;
}
