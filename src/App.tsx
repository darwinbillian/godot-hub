import RootLayout from "@/layouts/RootLayout";
import { show } from "@/lib/ipc/commands";
import InstallListPage from "@/pages/installs/InstallListPage";
import NewInstallPage from "@/pages/installs/NewInstallPage";
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
          <Route index element={<InstallListPage />} />
          <Route path="new" element={<NewInstallPage />} />
        </Route>
      </Route>
    </Routes>
  );
}
