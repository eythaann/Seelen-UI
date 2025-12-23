import { SeelenCommand } from "@seelen-ui/lib";
import { AnimatedDropdown } from "@shared/components/AnimatedWrappers";
import { FileIcon } from "libs/ui/react/components/Icon/index.tsx";
import { OverflowTooltip } from "libs/ui/react/components/OverflowTooltip/index.tsx";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Menu } from "antd";
import { memo } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import type { StartMenuApp } from "../../shared/store/domain.ts";

export const Item = memo(
  ({ item, hidden }: { item: StartMenuApp; hidden: boolean }) => {
    const { path, umid } = item;

    const { t } = useTranslation();

    function onClick() {
      invoke(SeelenCommand.OpenFile, { path });
      getCurrentWindow().hide();
    }

    const parts = path.split("\\");
    const filename = parts.at(-1);

    const shortPath = path.slice(path.indexOf("\\Programs\\") + 10);
    const displayName = filename?.slice(0, filename.lastIndexOf(".")) ||
      filename || "";

    return (
      <AnimatedDropdown
        animationDescription={{
          openAnimationName: "launcher-item-context-menu-open",
          closeAnimationName: "launcher-item-context-menu-close",
        }}
        trigger={["contextMenu"]}
        dropdownRender={() => (
          <BackgroundByLayersV2
            className="launcher-item-context-menu"
            prefix="menu"
            onContextMenu={(e) => {
              e.stopPropagation();
              e.preventDefault();
            }}
          >
            <Menu
              items={[
                {
                  label: t("item.pin"),
                  key: "pin",
                  onClick() {
                    invoke(SeelenCommand.WegPinItem, { path });
                  },
                },
                {
                  label: t("item.open_location"),
                  key: "open",
                  onClick() {
                    invoke(SeelenCommand.SelectFileOnExplorer, { path });
                  },
                },
              ]}
            />
          </BackgroundByLayersV2>
        )}
      >
        <button
          style={{ display: hidden ? "none" : undefined }}
          className="launcher-item"
          onClick={onClick}
        >
          <FileIcon className="launcher-item-icon" path={path} umid={umid} />
          <OverflowTooltip className="launcher-item-label" text={displayName} />
          <OverflowTooltip className="launcher-item-path" text={shortPath} />
        </button>
      </AnimatedDropdown>
    );
  },
);
