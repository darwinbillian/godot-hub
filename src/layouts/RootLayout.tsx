import { Outlet } from "react-router";

export default function RootLayout() {
  return (
    <div className="min-h-screen bg-neutral-900 text-neutral-200">
      <Outlet />
    </div>
  );
}
