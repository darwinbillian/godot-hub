import { ArrowLeftIcon, ExternalLinkIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router";
import { install, listVersions, Version } from "../../lib/commands";

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
        <Link
          className="cursor-pointer rounded p-1 text-neutral-400 transition hover:bg-white/10 hover:text-neutral-200"
          to="/installs"
        >
          <ArrowLeftIcon size={20} />
        </Link>
        <h1 className="text-2xl font-semibold">Install Godot Editor</h1>
      </div>
      <div className="flex flex-col gap-4">
        {versions?.map((version) => (
          <VersionCard key={version.name} version={version} />
        ))}
      </div>
    </div>
  );
}

function VersionCard({ version }: { version: Version }) {
  const navigate = useNavigate();

  return (
    <div className="flex items-center gap-2 rounded border border-white/10 bg-neutral-800 p-4">
      <div className="flex flex-1 items-center gap-2">
        <img className="size-8" src="/icon.svg" />
        <div className="font-semibold">Godot {version.name}</div>
      </div>
      <div className="flex items-center gap-2">
        <a
          className="flex cursor-pointer items-center gap-1 text-sm text-neutral-400 transition hover:text-neutral-200"
          href={version.release_notes}
          target="_blank"
        >
          <span>Release notes</span>
          <ExternalLinkIcon size={16} />
        </a>
        <button
          className="cursor-pointer rounded bg-blue-500 px-2 py-1 font-semibold transition hover:bg-blue-600 disabled:cursor-default disabled:bg-neutral-800"
          disabled={version.installed}
          onClick={() => {
            install(version.name, version.flavor).catch((e) =>
              console.error(e),
            );

            navigate("/installs");
          }}
        >
          {version.installed ? "Installed" : "Install"}
        </button>
      </div>
    </div>
  );
}
