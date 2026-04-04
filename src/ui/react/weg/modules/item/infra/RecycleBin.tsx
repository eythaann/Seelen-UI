import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { SpecificIcon } from "libs/ui/react/components/Icon/index.tsx";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { TrashBinItem } from "../../shared/types.ts";

import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";
import { getMenuForItem } from "./GeneralMenu.tsx";
import { $trash_bin_info } from "../../shared/state/system.ts";

interface Props {
  item: TrashBinItem;
}

export const TrashBin = memo(({ item }: Props) => {
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
        invoke(SeelenCommand.OpenFile, { path: "shell:RecycleBinFolder" });
      }}
      onContextMenu={onContextMenu}
    >
      <SpecificIcon
        className="weg-item-icon"
        name={$trash_bin_info.value.itemCount > 0 ? "bin::full" : "bin::empty"}
      />
    </div>
  );
});
