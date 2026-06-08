import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExternalLinkIcon } from "lucide-react";
import "./App.css";

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

export default function App() {
  const [versions, setVersions] = useState<Version[]>();

  useEffect(() => {
    invoke<Version[]>("list_versions")
      .then((versions) => setVersions(versions))
      .catch((e) => console.error(e));
  }, []);

  return (
    <div className="min-h-screen bg-neutral-900 text-neutral-200">
      <div className="flex flex-col gap-8 p-8 ">
        <h1 className="text-2xl font-semibold">Install Godot Editor</h1>
        <div className="flex flex-col gap-4">
          {versions
            ?.filter((version) => version.flavor === "stable")
            .map((version) => (
              <VersionItem key={version.name} version={version} />
            ))}
        </div>
      </div>
    </div>
  );
}

function VersionItem({ version }: { version: Version }) {
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
        <button className="px-2 py-1 font-semibold bg-blue-500 rounded transition cursor-pointer hover:bg-blue-600">
          Install
        </button>
      </div>
    </div>
  );
}
