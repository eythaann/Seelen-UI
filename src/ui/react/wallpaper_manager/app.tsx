import { useEffect } from "react";

import { MonitorContainers } from "./modules/Monitor/infra.tsx";
import { Widget } from "@seelen-ui/lib";

export function App() {
  useEffect(() => {
    // We manually shows the widget to avoid the focus call on ready
    // as this is a unfocusable widget causes the wallpaper to not be correctly
    // positioned under the desktop
    Widget.self.ready({ show: false }).then(() => {
      Widget.self.show();
    });
  }, []);

  return <MonitorContainers />;
}
