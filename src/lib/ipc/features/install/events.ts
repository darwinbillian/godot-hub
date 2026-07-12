import { listen } from "@tauri-apps/api/event";
import {
  InstallAddEventArgs,
  InstallRemoveEventArgs,
  InstallUpdateEventArgs,
} from "./types";

export const addEvent = {
  subscribe: (handler: (args: InstallAddEventArgs) => void): (() => void) => {
    const unlisten = listen<InstallAddEventArgs>("installs::add", (event) => {
      handler(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  },
};

export const updateEvent = {
  subscribe: (
    handler: (args: InstallUpdateEventArgs) => void,
  ): (() => void) => {
    const unlisten = listen<InstallUpdateEventArgs>(
      "installs::update",
      (event) => {
        handler(event.payload);
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  },
};

export const removeEvent = {
  subscribe: (
    handler: (args: InstallRemoveEventArgs) => void,
  ): (() => void) => {
    const unlisten = listen<InstallRemoveEventArgs>(
      "installs::remove",
      (event) => {
        handler(event.payload);
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  },
};
