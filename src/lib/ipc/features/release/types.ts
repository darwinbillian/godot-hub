import { Install } from "@/lib/ipc/features/install/types";

export interface Release {
  name: string;
  flavor: string;
  release_notes: string;
  status: ReleaseStatus;
  install?: Install;
}

export type ReleaseStatus = { type: "available" };
