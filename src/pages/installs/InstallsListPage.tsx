import {
  launch,
  list,
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

export default function InstallsListPage() {
  const [installs, setInstalls] = useState<Install[]>();

  useEffect(() => {
    list()
      .then((installs) => setInstalls(installs))
      .catch((e) => console.error(e));
  }, []);

  useEffect(() => {
    const unlisten = listen<InstallUpdateEventArgs>(
      "installs::update",
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
      "installs::remove",
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
          <Link className="btn btn-primary" to="/installs/install">
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
      <div>
        <img className="size-8" src="/icon.svg" />
      </div>
      <div className="flex-1">
        <h2 className="font-semibold">Godot {install.version}</h2>
        <InstallCardBody install={install} />
      </div>
      <div>
        <InstallCardActions install={install} />
      </div>
    </div>
  );
});

function InstallCardBody({ install }: { install: Install }) {
  const renderProgress = () => {
    if (install.status.type !== "installing") {
      return null;
    }

    switch (install.status.progress.type) {
      case "starting":
        return <>Starting...</>;
      case "downloading":
        return <>Downloading...</>;
      case "extracting":
        return <>Extracting...</>;
      case "finalizing":
        return <>Finalizing...</>;
      default:
        return null;
    }
  };

  const renderContent = () => {
    switch (install.status.type) {
      case "installing":
        return <>{renderProgress()}</>;
      case "installed":
        return <>{install.status.installation.dir}</>;
      case "failed":
        return (
          <details>
            <summary>Failed</summary>
            <p className="text-red-400">{install.status.error.message}</p>
          </details>
        );
      default:
        return null;
    }
  };

  return <div className="text-sm text-neutral-400">{renderContent()}</div>;
}

function InstallCardActions({ install }: { install: Install }) {
  const [expand, setExpand] = useState(false);

  if (install.status.type !== "installed") {
    return null;
  }

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
