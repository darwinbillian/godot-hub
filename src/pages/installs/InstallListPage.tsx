import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ChevronDownIcon, PlayIcon, Trash2Icon } from "lucide-react";
import { useEffect, useState } from "react";
import { Link } from "react-router";

interface Install {
  id: string;
  dir: string;
  version: string;
  flavor: string;
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
          <InstallItem install={install} key={install.id} />
        ))}
      </div>
    </div>
  );
}

function InstallItem({ install }: { install: Install }) {
  const [expand, setExpand] = useState(false);

  return (
    <div className="flex gap-2 p-4 border border-white/10 bg-neutral-800 rounded">
      <div className="flex flex-1 gap-2">
        <img className="size-8" src="/icon.svg" />
        <div>
          <div className="font-semibold">Godot {install.version}</div>
          <div className="text-sm text-neutral-400">{install.dir}</div>
        </div>
      </div>
      <div>
        <div className="relative flex items-stretch">
          <button
            className="flex items-center gap-1 px-2 py-1 font-semibold bg-blue-500 rounded-l transition cursor-pointer hover:bg-blue-600"
            onClick={() => {
              invoke("launch", { id: install.id }).catch((e) =>
                console.error(e),
              );
            }}
          >
            <PlayIcon size={16} />
            Launch
          </button>
          <button
            className="p-1 bg-blue-500 rounded-r transition cursor-pointer hover:bg-blue-600"
            onClick={() => {
              setExpand((expand) => !expand);
            }}
          >
            <ChevronDownIcon size={16} />
          </button>
          {expand && (
            <>
              <div
                className="fixed inset-0 z-10"
                onClick={() => {
                  setExpand(false);
                }}
              />
              <div className="absolute z-10 top-full right-0 w-max flex flex-col p-2 border border-white/10 bg-neutral-800 rounded">
                <button
                  className="flex items-center gap-2 px-2 py-1 bg-neutral-800 rounded transition cursor-pointer hover:bg-neutral-700"
                  onClick={() => {
                    invoke("uninstall", { id: install.id }).catch((e) =>
                      console.error(e),
                    );

                    setExpand(false);
                  }}
                >
                  <Trash2Icon size={16} />
                  Uninstall
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
