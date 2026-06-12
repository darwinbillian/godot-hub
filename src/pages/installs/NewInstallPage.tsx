import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeftIcon, ExternalLinkIcon } from "lucide-react";
import { Link, useNavigate } from "react-router";

interface Version {
  name: string;
  flavor: string;
  release_date: string;
  release_notes: string;
  featured?: string;
  releases?: VersionRelease[];
}

interface VersionRelease {
  name: string;
  release_date: string;
  release_notes: string;
  release_version?: string;
}

export default function NewInstallPage() {
  const [versions, setVersions] = useState<Version[]>();

  useEffect(() => {
    invoke<Version[]>("list_versions")
      .then((versions) => setVersions(versions))
      .catch((e) => console.error(e));
  }, []);

  return (
    <div className="flex flex-col gap-8 p-8">
      <div className="flex items-center gap-2">
        <Link
          className="p-1 rounded text-neutral-400 transition cursor-pointer hover:bg-neutral-800 hover:text-neutral-200"
          to="/installs"
        >
          <ArrowLeftIcon size={20} />
        </Link>
        <h1 className="text-2xl font-semibold">Install Godot Editor</h1>
      </div>
      <div className="flex flex-col gap-4">
        {versions
          ?.filter((version) => version.flavor === "stable")
          .map((version) => (
            <VersionItem key={version.name} version={version} />
          ))}
      </div>
    </div>
  );
}

function VersionItem({ version }: { version: Version }) {
  const navigate = useNavigate();

  return (
    <div className="flex items-center gap-2 p-4 border border-white/10 rounded">
      <div className="flex flex-1 items-center gap-2">
        <img className="size-8" src="/icon.svg" />
        <div className="font-semibold">Godot {version.name}</div>
      </div>
      <div className="flex items-center gap-2">
        <a
          className="flex items-center gap-1 text-sm text-neutral-400 transition cursor-pointer hover:text-neutral-200"
          href={"https://godotengine.org" + version.release_notes}
          target="_blank"
        >
          <span>Release notes</span>
          <ExternalLinkIcon size={16} />
        </a>
        <button
          className="px-2 py-1 font-semibold bg-blue-500 rounded transition cursor-pointer hover:bg-blue-600"
          onClick={() => {
            invoke("install", {
              version: version.name,
              flavor: version.flavor,
            }).catch((e) => console.error(e));

            navigate("/installs");
          }}
        >
          Install
        </button>
      </div>
    </div>
  );
}
