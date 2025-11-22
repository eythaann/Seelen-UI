import { SeelenCommand } from "@seelen-ui/lib";
import { SpecificIcon } from "@shared/components/Icon";
import { invoke } from "@tauri-apps/api/core";
import { memo } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import type { StartMenuWegItem } from "../../shared/types.ts";

import { WithContextMenu } from "../../../components/WithContextMenu.tsx";
import { getMenuForItem } from "./Menu.tsx";
import { $delayedFocused } from "../../shared/state/windows.ts";

interface Props {
  item: StartMenuWegItem;
}

const startMenuExes = ["SearchHost.exe", "StartMenuExperienceHost.exe"];

export const StartMenu = memo(({ item }: Props) => {
  const { t } = useTranslation();

  const isStartMenuOpen = startMenuExes.some((program) => ($delayedFocused.value?.exe || "").endsWith(program));

  return (
    <WithContextMenu items={getMenuForItem(t, item)}>
      <div
        className="weg-item weg-item-start"
        onClick={() => {
          if (!isStartMenuOpen) {
            invoke(SeelenCommand.SendKeys, { keys: "{win}" });
          }
        }}
        onContextMenu={(e) => e.stopPropagation()}
      >
        <BackgroundByLayersV2 />
        <SpecificIcon
          className="weg-item-icon weg-item-start-icon"
          name="@seelen/weg::start-menu"
        />
      </div>
    </WithContextMenu>
  );
});
