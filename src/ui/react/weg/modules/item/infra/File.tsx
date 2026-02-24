import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { FileIcon, SpecificIcon } from "libs/ui/react/components/Icon/index.tsx";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import type { PinnedWegItem } from "../../shared/types.ts";

import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";
import { getMenuForItem } from "./Menu.tsx";

interface Props {
  item: PinnedWegItem;
}

export const FileOrFolder = memo(({ item }: Props) => {
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
      className="weg-item"
      onClick={() => {
        invoke(SeelenCommand.OpenFile, { path: item.path });
      }}
      onContextMenu={onContextMenu}
    >
      <BackgroundByLayersV2 prefix="item" />
      {item.subtype === "Folder"
        ? (
          <SpecificIcon
            className="weg-item-icon weg-item-folder-icon"
            name="@seelen/weg::folder"
          />
        )
        : (
          <FileIcon
            className="weg-item-icon"
            path={item.path}
            umid={item.umid}
          />
        )}
    </div>
  );
});
