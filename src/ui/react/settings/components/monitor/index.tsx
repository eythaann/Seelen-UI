import { Wallpaper } from "@shared/components/Wallpaper";
import { useSelector } from "react-redux";

import { newSelectors } from "../../modules/shared/store/app/reducer.ts";

import cs from "./index.module.css";
import { $virtual_desktops } from "../../modules/shared/signals.ts";

interface Props {
  monitorId: string;
  width?: number;
  height?: number;
}

export function Monitor({ monitorId, width = 1920, height = 1080 }: Props) {
  const wallpapers = useSelector(newSelectors.wallpapers);
  const wallpaperSettings = useSelector(newSelectors.byWallpaper);

  const monitor = $virtual_desktops.value.monitors[monitorId];
  const workspace = monitor?.workspaces.find((w) => w.id === monitor.active_workspace);

  const wallpaperId = workspace?.wallpaper;
  const wallpaper = wallpapers.find((w) => w.id === wallpaperId);

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
