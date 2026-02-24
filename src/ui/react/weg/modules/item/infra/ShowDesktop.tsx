import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { SpecificIcon } from "libs/ui/react/components/Icon/index.tsx";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import type { ShowDesktopWegItem } from "../../shared/types.ts";

import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";
import { getMenuForItem } from "./Menu.tsx";

interface Props {
  item: ShowDesktopWegItem;
}

export const ShowDesktopModule = memo(({ item }: Props) => {
  const { t } = useTranslation();

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
      className="weg-item weg-item-show-desktop"
      onClick={() => {
        invoke(SeelenCommand.ShowDesktop);
      }}
      onContextMenu={onContextMenu}
    >
      <BackgroundByLayersV2 />
      <SpecificIcon className="weg-item-icon" name="@seelen/weg::show-desktop" />
    </div>
  );
});
