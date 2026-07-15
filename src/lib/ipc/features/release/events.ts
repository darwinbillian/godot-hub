import { listen } from "@tauri-apps/api/event";
import { ReleaseUpdateEventArgs } from "./types";

export const updateEvent = {
  subscribe: (
    handler: (args: ReleaseUpdateEventArgs) => void,
  ): (() => void) => {
    const unlisten = listen<ReleaseUpdateEventArgs>(
      "releases::update",
      (event) => {
        handler(event.payload);
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  },
};
