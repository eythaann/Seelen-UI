import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { SpecificIcon } from "libs/ui/react/components/Icon/index.tsx";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import type { StartMenuWegItem } from "../../shared/types.ts";

import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";
import { getMenuForItem } from "./Menu.tsx";
import { $delayedFocused } from "../../shared/state/windows.ts";

interface Props {
  item: StartMenuWegItem;
}

const startMenuExes = ["SearchHost.exe", "StartMenuExperienceHost.exe"];

export const StartMenu = memo(({ item }: Props) => {
  const { t } = useTranslation();

  const isStartMenuOpen = startMenuExes.some((program) => ($delayedFocused.value?.exe || "").endsWith(program));

  const onContextMenu = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      const { alignX, alignY } = getDockContextMenuAlignment($settings.value.position);
      invoke(SeelenCommand.TriggerContextMenu, {
        menu: { ...getMenuForItem(t, item), alignX, alignY },
        forwardTo: null,
      });
    },
    [item, t],
  );

  return (
    <div
      className="weg-item weg-item-start"
      onClick={() => {
        if (!isStartMenuOpen) {
          invoke(SeelenCommand.ShowStartMenu);
        }
      }}
      onContextMenu={onContextMenu}
    >
      <BackgroundByLayersV2 />
      <SpecificIcon
        className="weg-item-icon weg-item-start-icon"
        name="@seelen/weg::start-menu"
      />
    </div>
  );
});
