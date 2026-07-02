import {
  launch,
  listInstalls,
  reveal,
  uninstall,
} from "@/lib/ipc/features/install/commands";
import {
  Install,
  InstallRemoveEventArgs,
  InstallUpdateEventArgs,
} from "@/lib/ipc/features/install/types";
import { listen } from "@tauri-apps/api/event";
import {
  ChevronDownIcon,
  FolderOpenIcon,
  PlayIcon,
  Trash2Icon,
} from "lucide-react";
import { memo, useEffect, useState } from "react";
import { Link } from "react-router";

export default function InstallListPage() {
  const [installs, setInstalls] = useState<Install[]>();

  useEffect(() => {
    listInstalls()
      .then((installs) => setInstalls(installs))
      .catch((e) => console.error(e));
  }, []);

  useEffect(() => {
    const unlisten = listen<InstallUpdateEventArgs>(
      "update_install",
      (event) => {
        setInstalls(
          (installs) =>
            installs &&
            installs.map((install) =>
              event.payload.id === install.id
                ? { ...install, status: event.payload.status }
                : install,
            ),
        );
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<InstallRemoveEventArgs>(
      "remove_install",
      (event) => {
        setInstalls(
          (installs) =>
            installs &&
            installs.filter((install) => event.payload.id !== install.id),
        );
      },
    );

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
          <Link className="btn btn-primary" to="/installs/new">
            Install Editor
          </Link>
        </div>
      </div>
      <div>
        <ul className="list gap-4">
          {installs?.map((install) => (
            <li key={install.id}>
              <InstallCard install={install} />
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

const InstallCard = memo(({ install }: { install: Install }) => {
  return (
    <div className="card flex gap-2 p-4">
      <div className="flex flex-1 gap-2">
        <img className="size-8" src="/icon.svg" />
        <div>
          <div className="font-semibold">Godot {install.version}</div>
          <div className="text-sm text-neutral-400">
            {install.status.type === "installed" ? (
              <span>{install.status.installation.dir}</span>
            ) : install.status.type === "failed" ? (
              <details>
                <summary>Failed</summary>
                <p className="text-red-400">{install.status.error.message}</p>
              </details>
            ) : (
              <span>In progress</span>
            )}
          </div>
        </div>
      </div>
      {install.status.type === "installed" && (
        <div>
          <InstallButton install={install} />
        </div>
      )}
    </div>
  );
});

function InstallButton({ install }: { install: Install }) {
  const [expand, setExpand] = useState(false);

  return (
    <div className="relative flex items-stretch">
      <button
        className="btn btn-primary rounded-r-none"
        onClick={() => {
          launch(install.id).catch((e) => console.error(e));
        }}
      >
        <PlayIcon size={16} />
        Launch
      </button>
      <button
        className="btn btn-primary rounded-l-none p-1"
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
          <div className="absolute top-full right-0 z-10 w-max">
            <ul className="menu">
              <li>
                <button
                  onClick={() => {
                    reveal(install.id).catch((e) => console.error(e));

                    setExpand(false);
                  }}
                >
                  <FolderOpenIcon size={16} />
                  Show in Explorer
                </button>
              </li>
              <li>
                <button
                  onClick={() => {
                    uninstall(install.id).catch((e) => console.error(e));

                    setExpand(false);
                  }}
                >
                  <Trash2Icon size={16} />
                  Uninstall
                </button>
              </li>
            </ul>
          </div>
        </>
      )}
    </div>
  );
}
