import { SeelenWegMode, SeelenWegSide } from "@seelen-ui/lib/types";
import { cx } from "@shared/styles";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { WithContextMenu } from "../../components/WithContextMenu.tsx";
import { $dock_should_be_hidden, $settings } from "../shared/state/mod.ts";
import { DockItems } from "./ItemReordableList.tsx";
import { getSeelenWegMenu } from "./menu.tsx";

export function SeelenWeg() {
  const { t } = useTranslation();

  const settings = $settings.value;
  const isHorizontal = settings.position === SeelenWegSide.Top ||
    settings.position === SeelenWegSide.Bottom;

  return (
    <WithContextMenu items={getSeelenWegMenu(t)}>
      <div
        className={cx("taskbar", settings.position.toLowerCase(), {
          horizontal: isHorizontal,
          vertical: !isHorizontal,
          // 'temporal-only': isTemporalOnlyWegBar, todo handle this type of state via new config
          "full-width": settings.mode === SeelenWegMode.FullWidth,
          hidden: $dock_should_be_hidden.value,
        })}
      >
        <BackgroundByLayersV2 prefix="taskbar" />
        <div className="weg-items-container">
          <DockItems isHorizontal={isHorizontal} />
        </div>
      </div>
    </WithContextMenu>
  );
}
