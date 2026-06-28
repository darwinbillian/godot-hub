import { install, listVersions, Version } from "@/lib/commands";
import {
  ArrowLeftIcon,
  ExternalLinkIcon,
  OctagonAlertIcon,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router";

export default function NewInstallPage() {
  const [versions, setVersions] = useState<Version[]>();

  useEffect(() => {
    listVersions()
      .then((versions) => setVersions(versions))
      .catch((e) => console.error(e));
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
        <ul className="list gap-4">
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

function VersionCard({ version }: { version: Version }) {
  const navigate = useNavigate();

  return (
    <div className="card flex items-center gap-2 p-4">
      <div className="flex flex-1 items-center gap-2">
        <img className="size-8" src="/icon.svg" />
        <div className="font-semibold">Godot {version.name}</div>
      </div>
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
          <div title={version.status.error}>
            <OctagonAlertIcon className="text-red-400" />
          </div>
        )}
        <button
          className={
            version.status.type === "installing"
              ? "btn btn-outline"
              : version.status.type === "installed"
                ? "btn btn-disabled"
                : version.status.type === "failed"
                  ? "btn bg-neutral-700 hover:bg-neutral-600"
                  : "btn btn-primary"
          }
          disabled={
            version.status.type === "installing" ||
            version.status.type === "installed"
          }
          onClick={() => {
            install(version.name, version.flavor).catch((e) =>
              console.error(e),
            );

            navigate("/installs");
          }}
        >
          {version.status.type === "installing"
            ? "In Progress"
            : version.status.type === "installed"
              ? "Installed"
              : version.status.type === "failed"
                ? "Retry"
                : "Install"}
        </button>
      </div>
    </div>
  );
}
