import { install, resume } from "@/lib/ipc/features/install/commands";
import {
  addEvent,
  removeEvent,
  updateEvent,
} from "@/lib/ipc/features/install/events";
import { list } from "@/lib/ipc/features/release/commands";
import { Release } from "@/lib/ipc/features/release/types";
import {
  ArrowLeftIcon,
  ExternalLinkIcon,
  LoaderCircleIcon,
  OctagonAlertIcon,
  PlayIcon,
} from "lucide-react";
import { memo, useEffect, useState } from "react";
import { Link, useNavigate } from "react-router";

export default function InstallsInstallPage() {
  const [releases, setReleases] = useState<Release[]>();

  const updateReleases = () => {
    list()
      .then((releases) => setReleases(releases))
      .catch((e) => console.error(e));
  };

  useEffect(() => {
    updateReleases();
  }, []);

  useEffect(() => {
    return addEvent.subscribe(() => {
      updateReleases();
    });
  }, []);

  useEffect(() => {
    return updateEvent.subscribe((args) => {
      setReleases((releases) =>
        releases?.map((release) =>
          args.id === release.install?.id
            ? {
                ...release,
                install: {
                  ...release.install,
                  status: args.status,
                },
              }
            : release,
        ),
      );
    });
  }, []);

  useEffect(() => {
    return removeEvent.subscribe((args) => {
      setReleases((releases) =>
        releases?.map((release) =>
          args.id === release.install?.id
            ? {
                ...release,
                install: undefined,
              }
            : release,
        ),
      );
    });
  }, []);

  return (
    <div className="flex flex-col gap-8 p-8">
      <div className="flex items-center gap-2">
        <Link className="btn btn-ghost p-1" to="/installs">
          <ArrowLeftIcon size={20} />
        </Link>
        <h1 className="text-2xl font-semibold">Install Godot Editor</h1>
      </div>
      <div>
        <ul className="flex flex-col gap-4">
          {releases?.map((release) => (
            <li key={release.name}>
              <ReleaseCard release={release} />
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

const ReleaseCard = memo(({ release }: { release: Release }) => {
  return (
    <div className="card flex items-center gap-2 p-4">
      <div>
        <img className="size-8" src="/icon.svg" />
      </div>
      <div className="flex-1">
        <h2 className="font-semibold">Godot {release.name}</h2>
      </div>
      <div>
        <ReleaseCardActions release={release} />
      </div>
    </div>
  );
});

function ReleaseCardActions({ release }: { release: Release }) {
  const navigate = useNavigate();

  const renderButton = () => {
    if (release.install) {
      switch (release.install.status.type) {
        case "installing":
          return (
            <button className="btn btn-outline" disabled>
              <LoaderCircleIcon className="animate-spin" size={16} />
              In progress
            </button>
          );
        case "paused":
          const id = release.install.id;
          return (
            <button
              className="btn btn-primary"
              onClick={() => {
                resume(id)
                  .then(() => navigate("/installs"))
                  .catch((e) => console.error(e));
              }}
            >
              <PlayIcon size={16} />
              Resume download
            </button>
          );
        case "installed":
          return (
            <button className="btn btn-disabled" disabled>
              Installed
            </button>
          );
        case "failed":
          return (
            <>
              <div title={release.install.status.error.message}>
                <OctagonAlertIcon size={20} className="text-red-400" />
              </div>
              <Link
                className="btn bg-neutral-700 hover:bg-neutral-600"
                to="/installs"
              >
                See Details
              </Link>
            </>
          );
        default:
          return null;
      }
    } else {
      switch (release.status.type) {
        case "available":
          return (
            <button
              className="btn btn-primary"
              onClick={() => {
                install(release.name, release.flavor)
                  .then(() => navigate("/installs"))
                  .catch((e) => console.error(e));
              }}
            >
              Install
            </button>
          );
        default:
          return null;
      }
    }
  };

  return (
    <div className="flex items-center gap-2">
      <a
        className="btn btn-link text-sm"
        href={release.release_notes}
        target="_blank"
      >
        Release notes
        <ExternalLinkIcon size={16} />
      </a>
      {renderButton()}
    </div>
  );
}
