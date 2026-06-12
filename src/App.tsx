import { Navigate, Route, Routes } from "react-router";
import "./App.css";
import RootLayout from "./layouts/RootLayout";
import InstallListPage from "./pages/installs/InstallListPage";
import NewInstallPage from "./pages/installs/NewInstallPage";

export default function App() {
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
