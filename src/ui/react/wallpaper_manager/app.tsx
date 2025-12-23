import { useEffect } from "react";

import { MonitorContainers } from "./modules/Monitor/infra.tsx";
import { Widget } from "@seelen-ui/lib";

export function App() {
  useEffect(() => {
    Widget.getCurrent().ready();
  }, []);

  return <MonitorContainers />;
}
