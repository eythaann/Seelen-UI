import { SeelenWallWidgetId } from "@seelen-ui/lib";
import { WallpaperId } from "@seelen-ui/lib/types";
import { Wallpaper } from "@shared/components/Wallpaper";
import { useSelector } from "react-redux";

import { newSelectors } from "../../modules/shared/store/app/reducer";

import cs from "./index.module.css";

interface Props {
  monitorId: string;
  width?: number;
  height?: number;
}

export function Monitor({ monitorId, width = 1920, height = 1080 }: Props) {
  const wallpapers = useSelector(newSelectors.wallpapers);
  const wallpaperSettings = useSelector(newSelectors.byWallpaper);

  const baseEnabled = useSelector(newSelectors.wall.backgroundsV2);

  const configByMonitor = useSelector(newSelectors.monitorsV3);
  const monitorPatch = monitorId ? configByMonitor[monitorId] : null;

  const enabledOnMonitor = monitorPatch?.byWidget[SeelenWallWidgetId]
    ?.backgroundsV2 as WallpaperId[];
  const wallpaperId = enabledOnMonitor ? enabledOnMonitor[0] : baseEnabled[0];

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
      <div
        className={cs.monitor}
        style={style}
      >
        <div className={cs.screen}>
          <Wallpaper
            definition={wallpaper}
            config={wallpaperId ? wallpaperSettings[wallpaperId] : undefined}
          />
        </div>
      </div>
    </div>
  );
}
