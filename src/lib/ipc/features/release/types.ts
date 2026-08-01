import { Install } from "@/lib/ipc/features/install/types";

export interface Release {
  id: string;
  name: string;
  version: string;
  flavor: string;
  release_notes: string;
  status: ReleaseStatus;
  install?: Install;
}

export type ReleaseStatus = { type: "available" } | { type: "unavailable" };
