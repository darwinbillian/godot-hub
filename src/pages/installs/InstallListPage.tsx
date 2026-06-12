import { Link } from "react-router";

export default function InstallListPage() {
  return (
    <div className="p-8">
      <div className="flex items-center gap-2">
        <div className="flex-1">
          <h1 className="text-2xl font-semibold">Installs</h1>
        </div>
        <div>
          <Link
            className="px-2 py-1 font-semibold bg-blue-500 rounded transition cursor-pointer hover:bg-blue-600"
            to="/installs/new"
          >
            Install Editor
          </Link>
        </div>
      </div>
    </div>
  );
}
