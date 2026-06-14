import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { SeelenWegMode, SeelenWegSide } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useCallback } from "preact/compat";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import { $dock_should_be_hidden, $settings } from "../shared/state/index.ts";
import { getDockContextMenuAlignment } from "../shared/state/settings.ts";
import { DockItems } from "./ItemReordableList.tsx";
import { getSeelenWegMenu } from "./DockMenu.tsx";

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
      data-has-margin={!!settings.margin}
      data-size={settings.mode === SeelenWegMode.FullWidth ? "full-width" : "min-content"}
      className={cx("taskbar", settings.position.toLowerCase(), {
        horizontal: isHorizontal,
        vertical: !isHorizontal,
        hidden: $dock_should_be_hidden.value,
      })}
      onContextMenu={onContextMenu}
    >
      <BackgroundByLayersV2 />
      <div className="weg-items-container">
        <DockItems />
      </div>
    </div>
  );
}
