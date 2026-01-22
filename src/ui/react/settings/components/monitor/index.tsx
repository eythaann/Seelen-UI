import { Wallpaper } from "libs/ui/react/components/Wallpaper/index.tsx";

import cs from "./index.module.css";
import { $virtual_desktops } from "../../modules/shared/signals.ts";
import { wallpapers } from "../../state/resources.ts";
import { settings } from "../../state/mod.ts";

interface Props {
  monitorId: string;
  width?: number;
  height?: number;
}

export function Monitor({ monitorId, width = 1920, height = 1080 }: Props) {
  const wallpaperSettings = settings.value.byWallpaper;

  const monitor = $virtual_desktops.value.monitors[monitorId];
  const workspace = monitor?.workspaces.find((w) => w.id === monitor.active_workspace);

  const wallpaperId = workspace?.wallpaper;
  const wallpaper = wallpapers.value.find((w) => w.id === wallpaperId);

  const style: React.CSSProperties = {
    aspectRatio: `${width} / ${height}`,
  };
  if (width > height) {
    style.width = "100%";
  } else {
    style.height = "100%";
  }

  return (
    <div className={cs.monitorContainer}>
      <div className={cs.monitor} style={style}>
        <div className={cs.screen}>
          <Wallpaper
            definition={wallpaper}
            config={wallpaperId ? wallpaperSettings[wallpaperId] : undefined}
            muted
          />
        </div>
      </div>
    </div>
  );
}
