import { SeelenCommand } from "@seelen-ui/lib";
import { SpecificIcon } from "libs/ui/react/components/Icon/index.tsx";
import { invoke } from "@tauri-apps/api/core";
import { memo } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import type { ShowDesktopWegItem } from "../../shared/types.ts";

import { WithContextMenu } from "../../../components/WithContextMenu.tsx";
import { getMenuForItem } from "./Menu.tsx";

interface Props {
  item: ShowDesktopWegItem;
}

export const ShowDesktopModule = memo(({ item }: Props) => {
  const { t } = useTranslation();

  return (
    <WithContextMenu items={getMenuForItem(t, item)}>
      <div
        className="weg-item weg-item-show-desktop"
        onClick={() => {
          invoke(SeelenCommand.ShowDesktop);
        }}
        onContextMenu={(e) => e.stopPropagation()}
      >
        <BackgroundByLayersV2 />
        <SpecificIcon className="weg-item-icon" name="@seelen/weg::show-desktop" />
      </div>
    </WithContextMenu>
  );
});
