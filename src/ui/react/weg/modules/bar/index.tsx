import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { SeelenWegMode, SeelenWegSide } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useCallback } from "preact/compat";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import { $dock_should_be_hidden, $settings } from "../shared/state/mod.ts";
import { getDockContextMenuAlignment } from "../shared/state/settings.ts";
import { DockItems } from "./ItemReordableList.tsx";
import { getSeelenWegMenu } from "./menu.tsx";

export function SeelenWeg() {
  const { t } = useTranslation();

  const settings = $settings.value;
  const isHorizontal = settings.position === SeelenWegSide.Top ||
    settings.position === SeelenWegSide.Bottom;

  const onContextMenu = useCallback(() => {
    const { alignX, alignY } = getDockContextMenuAlignment(settings.position);
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getSeelenWegMenu(t), alignX, alignY },
      forwardTo: null,
    });
  }, [t, settings.position]);

  return (
    <div
      className={cx("taskbar", settings.position.toLowerCase(), {
        horizontal: isHorizontal,
        vertical: !isHorizontal,
        // 'temporal-only': isTemporalOnlyWegBar, todo handle this type of state via new config
        "full-width": settings.mode === SeelenWegMode.FullWidth,
        hidden: $dock_should_be_hidden.value,
      })}
      onContextMenu={onContextMenu}
    >
      <BackgroundByLayersV2 prefix="taskbar" />
      <div className="weg-items-container">
        <DockItems />
      </div>
    </div>
  );
}
