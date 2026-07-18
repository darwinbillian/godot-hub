import { Menu } from "@/components/Menu";
import { Modal } from "@/components/Modal";
import { Progress } from "@/components/Progress";
import {
  cancel,
  install,
  launch,
  list,
  pause,
  resume,
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
  ChevronRightIcon,
  FolderOpenIcon,
  HardDriveDownloadIcon,
  PauseIcon,
  PlayIcon,
  RotateCcwIcon,
  Trash2Icon,
  XIcon,
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
          <ul className="flex flex-col gap-4">
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
        <h2 className="font-semibold">{install.name}</h2>
        <InstallCardBody install={install} />
      </div>
      <div>
        <InstallCardActions install={install} />
      </div>
    </div>
  );
});

function InstallCardBody({ install }: { install: Install }) {
  const [detailsOpen, setDetailsOpen] = useState(false);

  const getProgress = () => {
    if (
      install.status.type !== "installing" &&
      install.status.type !== "paused"
    ) {
      return {
        text: <>In progress...</>,
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
          pausedText: <>Download paused ({Math.floor(percentage * 100)}%)</>,
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

  switch (install.status.type) {
    case "installing":
    case "paused":
      const { text, pausedText, percentage } = getProgress();
      return (
        <div className="flex flex-col gap-1">
          <div className="flex">
            <div className="flex-1">
              <p className="text-sm text-neutral-400">
                {install.status.type === "paused"
                  ? (pausedText ?? "Paused")
                  : text}
              </p>
            </div>
            <div>
              {install.status.type === "paused" ? (
                <button
                  className="btn btn-ghost p-1"
                  onClick={() => {
                    resume(install.id).catch((e) => console.error(e));
                  }}
                >
                  <PlayIcon size={16} />
                </button>
              ) : (
                <button
                  className="btn btn-ghost p-1"
                  onClick={() => {
                    pause(install.id).catch((e) => console.error(e));
                  }}
                >
                  <PauseIcon size={16} />
                </button>
              )}
              <CancelButton install={install} />
            </div>
          </div>
          <div>
            <Progress className="progress" value={percentage} />
          </div>
        </div>
      );
    case "installed":
      return (
        <p className="text-sm text-neutral-400">
          {install.status.installation.dir}
        </p>
      );
    case "failed":
      return (
        <div>
          <div className="flex items-center">
            <div className="flex flex-1 flex-col items-stretch">
              <button
                className="btn btn-link text-sm"
                onClick={() => {
                  setDetailsOpen((prev) => !prev);
                }}
              >
                {detailsOpen ? (
                  <ChevronDownIcon size={16} />
                ) : (
                  <ChevronRightIcon size={16} />
                )}
                Failed
              </button>
            </div>
            <div>
              <RetryButton install={install} />
              <CancelButton install={install} />
            </div>
          </div>
          {detailsOpen && (
            <div>
              <p className="text-sm text-red-400">
                {install.status.error.message}
              </p>
            </div>
          )}
        </div>
      );
    default:
      return null;
  }
}

function InstallCardActions({ install }: { install: Install }) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [uninstallModalOpen, setUninstallModalOpen] = useState(false);

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
          setMenuOpen(true);
        }}
      >
        <ChevronDownIcon size={16} />
      </button>
      <Menu
        open={menuOpen}
        onClose={() => {
          setMenuOpen(false);
        }}
      >
        <ul className="menu absolute top-full right-0 w-max">
          <li>
            <button
              onClick={() => {
                setMenuOpen(false);
                reveal(install.id).catch((e) => console.error(e));
              }}
            >
              <FolderOpenIcon size={16} />
              Show in Explorer
            </button>
          </li>
          <li>
            <button
              onClick={() => {
                setMenuOpen(false);
                setUninstallModalOpen(true);
              }}
            >
              <Trash2Icon size={16} />
              Uninstall
            </button>
          </li>
        </ul>
      </Menu>
      <Modal
        open={uninstallModalOpen}
        onClose={() => {
          setUninstallModalOpen(false);
        }}
      >
        <div className="modal w-120">
          <div className="flex items-center border-b">
            <div className="flex-1">
              <h2 className="text-lg font-semibold">Uninstall Editor</h2>
            </div>
            <div>
              <button
                className="btn btn-ghost p-1"
                onClick={() => {
                  setUninstallModalOpen(false);
                }}
              >
                <XIcon size={20} />
              </button>
            </div>
          </div>
          <div>
            <p>Are you sure you want to uninstall {install.name}?</p>
            <p>This action will remove the Editor from your system.</p>
          </div>
          <div className="flex justify-end gap-2 border-t">
            <button
              className="btn btn-outline"
              onClick={() => {
                setUninstallModalOpen(false);
              }}
            >
              Cancel
            </button>
            <button
              className="btn btn-error"
              onClick={() => {
                setUninstallModalOpen(false);
                uninstall(install.id).catch((e) => console.error(e));
              }}
            >
              Uninstall
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function RetryButton({ install: item }: { install: Install }) {
  return (
    <button
      className="btn btn-ghost p-1"
      title="Retry"
      onClick={() => {
        install(item.version, item.flavor).catch((e) => console.error(e));
      }}
    >
      <RotateCcwIcon size={16} />
    </button>
  );
}

function CancelButton({ install }: { install: Install }) {
  const [cancelModalOpen, setCancelModalOpen] = useState(false);

  return (
    <>
      <button
        className="btn btn-ghost p-1"
        title="Cancel"
        onClick={() => {
          setCancelModalOpen(true);
        }}
      >
        <XIcon size={16} />
      </button>
      <Modal
        open={cancelModalOpen}
        onClose={() => {
          setCancelModalOpen(false);
        }}
      >
        <div className="modal w-120">
          <div className="flex items-center border-b">
            <div className="flex-1">
              <h2 className="text-lg font-semibold">Cancel download?</h2>
            </div>
            <div>
              <button
                className="btn btn-ghost p-1"
                onClick={() => {
                  setCancelModalOpen(false);
                }}
              >
                <XIcon size={20} />
              </button>
            </div>
          </div>
          <div>
            <p>{install.name}</p>
          </div>
          <div className="flex justify-end gap-2 border-t">
            <button
              className="btn btn-error"
              onClick={() => {
                setCancelModalOpen(false);
                cancel(install.id).catch((e) => console.error(e));
              }}
            >
              Confirm
            </button>
          </div>
        </div>
      </Modal>
    </>
  );
}
