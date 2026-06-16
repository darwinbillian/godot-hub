import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Link } from "react-router";

interface Install {
  id: string;
  dir: string;
  metadata: InstallMetadata;
}

interface InstallMetadata {
  version: string;
  flavor: string;
  executable: string;
}

export default function InstallListPage() {
  const [installs, setInstalls] = useState<Install[]>();

  function updateInstalls() {
    invoke<Install[]>("list_installs")
      .then((installs) => setInstalls(installs))
      .catch((e) => console.error(e));
  }

  useEffect(() => {
    updateInstalls();
  }, []);

  useEffect(() => {
    const unlisten = listen("update_installs", () => {
      updateInstalls();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex flex-col gap-8 p-8">
      <div className="flex items-center gap-2">
        <div className="flex-1">
          <h1 className="text-2xl font-semibold">Installs</h1>
        </div>
        <div>
          <Link
            className="px-2 py-1 font-semibold bg-blue-500 rounded transition cursor-pointer hover:bg-blue-600"
            to="/installs/new"
          >
            Install Editor
          </Link>
        </div>
      </div>
      <div className="flex flex-col gap-4">
        {installs?.map((install) => (
          <InstallItem install={install} key={install.dir} />
        ))}
      </div>
    </div>
  );
}

function InstallItem({ install }: { install: Install }) {
  return (
    <div className="flex gap-2 p-4 border border-white/10 bg-neutral-800 rounded">
      <div className="flex flex-1 gap-2">
        <img className="size-8" src="/icon.svg" />
        <div>
          <div className="font-semibold">Godot {install.metadata.version}</div>
          <div className="text-sm text-neutral-400">{install.dir}</div>
        </div>
      </div>
      <div>
        <button
          className="px-2 py-1 font-semibold bg-blue-500 rounded transition cursor-pointer hover:bg-blue-600"
          onClick={() => {
            invoke("launch", { id: install.id }).catch((e) => console.error(e));
          }}
        >
          Launch
        </button>
      </div>
    </div>
  );
}
