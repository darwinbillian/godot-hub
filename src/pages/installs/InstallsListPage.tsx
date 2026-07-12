import { ProgressBar } from "@/components/ProgressBar";
import {
  launch,
  list,
  reveal,
  uninstall,
} from "@/lib/ipc/features/install/commands";
import {
  addEvent,
  removeEvent,
  updateEvent,
} from "@/lib/ipc/features/install/events";
import { Install } from "@/lib/ipc/features/install/types";
import {
  ChevronDownIcon,
  FolderOpenIcon,
  HardDriveDownloadIcon,
  PlayIcon,
  Trash2Icon,
} from "lucide-react";
import { memo, useEffect, useState } from "react";
import { Link } from "react-router";

export default function InstallsListPage() {
  const [installs, setInstalls] = useState<Install[]>();

  const updateInstalls = () => {
    list()
      .then((installs) => setInstalls(installs))
      .catch((e) => console.error(e));
  };

  useEffect(() => {
    updateInstalls();
  }, []);

  useEffect(() => {
    return addEvent.subscribe(() => {
      updateInstalls();
    });
  }, []);

  useEffect(() => {
    return updateEvent.subscribe((args) => {
      setInstalls((installs) =>
        installs?.map((install) =>
          args.id === install.id
            ? { ...install, status: args.status }
            : install,
        ),
      );
    });
  }, []);

  useEffect(() => {
    return removeEvent.subscribe((args) => {
      setInstalls((installs) =>
        installs?.filter((install) => args.id !== install.id),
      );
    });
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
        {installs?.length ? (
          <ul className="list gap-4">
            {installs.map((install) => (
              <li key={install.id}>
                <InstallCard install={install} />
              </li>
            ))}
          </ul>
        ) : (
          <div className="flex flex-col items-center gap-2 py-32 text-sm">
            <h2 className="font-semibold">No installs</h2>
            <p className="text-neutral-400">
              To get started, install a version of Godot Editor.
            </p>
            <Link
              className="btn btn-outline btn-primary"
              to="/installs/install"
            >
              <HardDriveDownloadIcon size={16} />
              Install Editor
            </Link>
          </div>
        )}
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
  const getProgress = () => {
    if (install.status.type !== "installing") {
      return {
        title: <>In progress...</>,
        percentage: 0,
      };
    }

    switch (install.status.progress.type) {
      case "starting":
        return {
          text: <>Starting...</>,
          percentage: 0,
        };
      case "downloading":
        const { downloaded, size } = install.status.progress.progress;
        const percentage = size ? downloaded / size : 0;
        return {
          text: <>Downloading... ({Math.floor(percentage * 100)}%)</>,
          percentage: percentage,
        };
      case "extracting":
        return {
          text: <>Extracting...</>,
          percentage: 1,
        };
      case "finalizing":
        return {
          text: <>Finalizing...</>,
          percentage: 1,
        };
      default:
        return {
          text: <>In progress...</>,
          percentage: 0,
        };
    }
  };

  const renderContent = () => {
    switch (install.status.type) {
      case "installing":
        let { text, percentage } = getProgress();
        return (
          <div className="flex flex-col gap-1">
            <div>{text}</div>
            <div>
              <ProgressBar value={percentage} />
            </div>
          </div>
        );
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
