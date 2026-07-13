import { install } from "@/lib/ipc/features/install/commands";
import { list } from "@/lib/ipc/features/version/commands";
import { updateEvent } from "@/lib/ipc/features/version/events";
import { Version } from "@/lib/ipc/features/version/types";
import {
  ArrowLeftIcon,
  ExternalLinkIcon,
  OctagonAlertIcon,
} from "lucide-react";
import { memo, useEffect, useState } from "react";
import { Link, useNavigate } from "react-router";

export default function InstallsInstallPage() {
  const [versions, setVersions] = useState<Version[]>();

  useEffect(() => {
    list()
      .then((versions) => setVersions(versions))
      .catch((e) => console.error(e));
  }, []);

  useEffect(() => {
    return updateEvent.subscribe((args) => {
      setVersions((versions) =>
        versions?.map((version) =>
          args.name === version.name && args.flavor === version.flavor
            ? { ...version, status: args.status }
            : version,
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
          {versions?.map((version) => (
            <li key={version.name}>
              <VersionCard version={version} />
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

const VersionCard = memo(({ version }: { version: Version }) => {
  return (
    <div className="card flex items-center gap-2 p-4">
      <div>
        <img className="size-8" src="/icon.svg" />
      </div>
      <div className="flex-1">
        <h2 className="font-semibold">Godot {version.name}</h2>
      </div>
      <div>
        <VersionCardActions version={version} />
      </div>
    </div>
  );
});

function VersionCardActions({ version }: { version: Version }) {
  const navigate = useNavigate();

  const handleInstall = () => {
    install(version.name, version.flavor).catch((e) => console.error(e));

    navigate("/installs");
  };

  const renderButton = () => {
    switch (version.status.type) {
      case "available":
        return (
          <button className="btn btn-primary" onClick={handleInstall}>
            Install
          </button>
        );
      case "installing":
        return (
          <button className="btn btn-outline" disabled>
            In progress
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
          <button
            className="btn bg-neutral-700 hover:bg-neutral-600"
            onClick={handleInstall}
          >
            Retry
          </button>
        );
      default:
        return null;
    }
  };

  return (
    <div className="flex items-center gap-2">
      <a
        className="btn btn-link text-sm"
        href={version.release_notes}
        target="_blank"
      >
        <span>Release notes</span>
        <ExternalLinkIcon size={16} />
      </a>
      {version.status.type === "failed" && (
        <div title={version.status.error.message}>
          <OctagonAlertIcon className="text-red-400" />
        </div>
      )}
      {renderButton()}
    </div>
  );
}
