import RootLayout from "@/layouts/RootLayout";
import { show } from "@/lib/ipc/commands";
import InstallsInstallPage from "@/pages/installs/InstallsInstallPage";
import InstallsListPage from "@/pages/installs/InstallsListPage";
import { useEffect } from "react";
import { Navigate, Route, Routes } from "react-router";
import "./App.css";

export default function App() {
  useEffect(() => {
    show();
  }, []);

  return (
    <Routes>
      <Route element={<RootLayout />}>
        <Route index element={<Navigate to="/installs" />} />
        <Route path="installs">
          <Route index element={<InstallsListPage />} />
          <Route path="install" element={<InstallsInstallPage />} />
        </Route>
      </Route>
    </Routes>
  );
}
