import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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
    <main>
      <h1>Godot Hub</h1>
      <div>
        {versions
          ?.filter((version) => version.flavor === "stable")
          .map((version) => (
            <div key={version.name}>{version.name}</div>
          ))}
      </div>
    </main>
  );
}
