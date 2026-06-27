import { listen } from "@tauri-apps/api/event";
import {
  ChevronDownIcon,
  FolderOpenIcon,
  PlayIcon,
  Trash2Icon,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Link } from "react-router";
import {
  Install,
  launch,
  listInstalls,
  reveal,
  uninstall,
} from "../../lib/commands";

export default function InstallListPage() {
  const [installs, setInstalls] = useState<Install[]>();

  function updateInstalls() {
    listInstalls()
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
            className="cursor-pointer rounded bg-blue-500 px-2 py-1 font-semibold transition hover:bg-blue-600"
            to="/installs/new"
          >
            Install Editor
          </Link>
        </div>
      </div>
      <div className="flex flex-col gap-4">
        {installs?.map((install) => (
          <InstallCard key={install.id} install={install} />
        ))}
      </div>
    </div>
  );
}

function InstallCard({ install }: { install: Install }) {
  return (
    <div className="flex gap-2 rounded border border-white/10 bg-neutral-800 p-4">
      <div className="flex flex-1 gap-2">
        <img className="size-8" src="/icon.svg" />
        <div>
          <div className="font-semibold">Godot {install.version}</div>
          <div className="text-sm text-neutral-400">{install.dir}</div>
        </div>
      </div>
      <div>
        <InstallButton install={install} />
      </div>
    </div>
  );
}

function InstallButton({ install }: { install: Install }) {
  const [expand, setExpand] = useState(false);

  return (
    <div className="relative flex items-stretch">
      <button
        className="flex cursor-pointer items-center gap-1 rounded-l bg-blue-500 px-2 py-1 font-semibold transition hover:bg-blue-600"
        onClick={() => {
          launch(install.id).catch((e) => console.error(e));
        }}
      >
        <PlayIcon size={16} />
        Launch
      </button>
      <button
        className="cursor-pointer rounded-r bg-blue-500 p-1 transition hover:bg-blue-600"
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
          <div className="absolute top-full right-0 z-10 flex w-max flex-col rounded border border-neutral-700 bg-neutral-800 p-2">
            <button
              className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 transition hover:bg-white/10"
              onClick={() => {
                reveal(install.id).catch((e) => console.error(e));

                setExpand(false);
              }}
            >
              <FolderOpenIcon size={16} />
              Show in Explorer
            </button>
            <button
              className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 transition hover:bg-white/10"
              onClick={() => {
                uninstall(install.id).catch((e) => console.error(e));

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
  );
}
