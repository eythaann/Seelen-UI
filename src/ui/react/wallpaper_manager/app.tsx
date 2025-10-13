import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import { MonitorContainers } from "./modules/Monitor/infra.tsx";

export function App() {
  useEffect(() => {
    getCurrentWindow().show();
  }, []);

  return <MonitorContainers />;
}
