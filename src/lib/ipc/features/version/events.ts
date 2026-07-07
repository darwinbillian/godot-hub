import { listen } from "@tauri-apps/api/event";
import { VersionUpdateEventArgs } from "./types";

export const updateEvent = {
  subscribe: (
    handler: (args: VersionUpdateEventArgs) => void,
  ): (() => void) => {
    const unlisten = listen<VersionUpdateEventArgs>(
      "versions::update",
      (event) => {
        handler(event.payload);
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  },
};
