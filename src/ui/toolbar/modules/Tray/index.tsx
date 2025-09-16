import { SeelenCommand } from "@seelen-ui/lib";
import { TrayIcon, TrayToolbarItem } from "@seelen-ui/lib/types";
import { AnimatedPopover } from "@shared/components/AnimatedWrappers";
import { FileIcon } from "@shared/components/Icon";
import { OverflowTooltip } from "@shared/components/OverflowTooltip";
import { useInterval, useWindowFocusChange } from "@shared/hooks";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { Item } from "../item/infra/infra";
import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { Selectors } from "../shared/store/app";

interface Props {
  module: TrayToolbarItem;
}

function TrayItem(props: { tray: TrayIcon }) {
  const [disabled, setDisabled] = useState(false);
  const { tray } = props;

  const { t } = useTranslation();

  let base64Icon: string | null = null;
  if (
    tray.registry.executablePath.endsWith("explorer.exe") &&
    tray.registry.iconSnapshot
  ) {
    const base64String = btoa(
      String.fromCharCode(...tray.registry.iconSnapshot),
    );
    base64Icon = `data:image/png;base64,${base64String}`;
  }

  return (
    <li
      className="tray-item"
      onClick={() => {
        if (!disabled) {
          invoke(SeelenCommand.OnClickTrayIcon, { key: tray.registry.key })
            .finally(() => {
              setDisabled(false);
            });
        }
        setDisabled(true);
      }}
      onContextMenu={() => {
        if (!disabled) {
          invoke(SeelenCommand.OnContextMenuTrayIcon, {
            key: tray.registry.key,
          }).finally(() => {
            setDisabled(false);
          });
        }
        setDisabled(true);
      }}
    >
      <div className="tray-item-icon-container">
        {base64Icon ? <img className="tray-item-icon" src={base64Icon} /> : (
          <FileIcon
            className="tray-item-icon"
            path={tray.registry.executablePath}
          />
        )}
      </div>
      <OverflowTooltip
        rootClassName="tray-item-label-tooltip"
        className="tray-item-label"
        text={tray.label ||
          tray.registry.initialTooltip ||
          tray.registry.executablePath.split("\\").pop() ||
          t("tray.unlabeled")}
        placement="left"
        arrow={false}
      />
    </li>
  );
}

export function TrayModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);

  const trayList = useSelector(Selectors.systemTray);
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  useInterval(
    () => {
      if (openPreview) {
        emit("hidden::tray-force-refresh");
      }
    },
    1000,
    [openPreview],
  );

  return (
    <AnimatedPopover
      animationDescription={{
        openAnimationName: "tray-open",
        closeAnimationName: "tray-close",
      }}
      trigger="click"
      open={openPreview}
      onOpenChange={(open) => {
        if (open) emit("hidden::tray-force-refresh");
        setOpenPreview(open);
      }}
      content={
        <BackgroundByLayersV2
          className="tray"
          prefix="tray"
          onContextMenu={(e) => e.stopPropagation()}
        >
          <ul className="tray-list">
            {trayList.map((tray) => <TrayItem key={tray.registry.key} tray={tray} />)}
          </ul>
          {trayList.length === 0 && <div className="tray-empty">{t("tray.empty")}</div>}
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} active={openPreview} />
    </AnimatedPopover>
  );
}
