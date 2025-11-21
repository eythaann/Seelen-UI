import { SeelenCommand } from "@seelen-ui/lib";
import { AnimatedPopover } from "@shared/components/AnimatedWrappers";
import { FileIcon } from "@shared/components/Icon";
import { useWindowFocusChange } from "@shared/hooks";
import { cx } from "@shared/styles";
import { invoke } from "@tauri-apps/api/core";
import moment from "moment";
import { memo, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { Selectors } from "../../shared/store/app.ts";

import type { PinnedWegItem, TemporalWegItem } from "../../shared/store/domain.ts";

import { WithContextMenu } from "../../../components/WithContextMenu.tsx";
import { $settings } from "../../shared/state/mod.ts";
import { getUserApplicationContextMenu } from "./UserApplicationContextMenu.tsx";
import { UserApplicationPreview } from "./UserApplicationPreview.tsx";
import { SeelenWegSide } from "node_modules/@seelen-ui/lib/esm/gen/types/SeelenWegSide";

interface Props {
  item: PinnedWegItem | TemporalWegItem;
  isOverlay?: boolean;
}

export const UserApplication = memo(({ item, isOverlay }: Props) => {
  const [openPreview, setOpenPreview] = useState(false);
  const [openContextMenu, setOpenContextMenu] = useState(false);
  const [blockUntil, setBlockUntil] = useState(moment(new Date()));

  const notifications = useSelector(Selectors.notifications);
  const devTools = useSelector(Selectors.devTools);
  const focusedApp = useSelector(Selectors.focusedApp);

  const { t } = useTranslation();
  const calculatePlacement = (position: any) => {
    switch (position) {
      case SeelenWegSide.Bottom: {
        return "top";
      }
      case SeelenWegSide.Top: {
        return "bottom";
      }
      case SeelenWegSide.Left: {
        return "right";
      }
      case SeelenWegSide.Right: {
        return "left";
      }
      default: {
        throw new Error("Not Implemented!");
      }
    }
  };

  useWindowFocusChange((focused) => {
    if (!focused) {
      setBlockUntil(moment(new Date()).add(1, "second"));
      setOpenPreview(false);
      setOpenContextMenu(false);
    }
  });

  useEffect(() => {
    if (openPreview && $settings.value.thumbnailGenerationEnabled) {
      invoke(SeelenCommand.WegRequestUpdatePreviews, {
        handles: item.windows.map((w) => w.handle),
      });
    }
  }, [openPreview]);

  const notificationsCount = notifications.filter((n) => n.appUmid === item.umid).length;
  const itemLabel = $settings.value.showWindowTitle && item.windows.length ? item.windows[0]!.title : null;

  const itemNode = (
    <div
      className="weg-item"
      onClick={() => {
        let window = item.windows[0];
        if (!window) {
          invoke(SeelenCommand.Run, {
            program: item.relaunchProgram,
            args: item.relaunchArgs,
            workingDir: item.relaunchIn,
          });
        } else {
          const wasFocused = focusedApp?.hwnd === window.handle;
          invoke(SeelenCommand.WegToggleWindowState, {
            hwnd: window.handle,
            wasFocused,
          });
        }
      }}
      onAuxClick={(e) => {
        let window = item.windows[0];
        if (e.button === 1 && window) {
          invoke(SeelenCommand.WegCloseApp, { hwnd: window.handle });
        }
      }}
    >
      <BackgroundByLayersV2 prefix="item" />
      <FileIcon className="weg-item-icon" path={item.path} umid={item.umid} />
      {itemLabel && <div className="weg-item-title">{itemLabel}</div>}
      {notificationsCount > 0 && <div className="weg-item-notification-badge">{notificationsCount}</div>}
      {$settings.value.showInstanceCounter && item.windows.length > 1 && (
        <div className="weg-item-instance-counter-badge">
          {item.windows.length}
        </div>
      )}
      {!$settings.value.showWindowTitle && (
        <div
          className={cx("weg-item-open-sign", {
            "weg-item-open-sign-active": !!item.windows.length,
            "weg-item-open-sign-focused": item.windows.some((w) => w.handle === focusedApp?.hwnd),
          })}
        />
      )}
    </div>
  );

  if (isOverlay) {
    return itemNode;
  }

  return (
    <WithContextMenu
      items={getUserApplicationContextMenu(
        t,
        item,
        devTools,
        $settings.value.showEndTask,
      ) || []}
      onOpenChange={(isOpen) => {
        setOpenContextMenu(isOpen);
        if (openPreview && isOpen) {
          setOpenPreview(false);
        }
      }}
    >
      <AnimatedPopover
        animationDescription={{
          openAnimationName: "weg-item-preview-container-open",
          closeAnimationName: "weg-item-preview-container-close",
        }}
        open={openPreview}
        placement={calculatePlacement($settings.value.position)}
        onOpenChange={(open) =>
          setOpenPreview(
            open && !openContextMenu && moment(new Date()) > blockUntil,
          )}
        trigger="hover"
        content={
          <BackgroundByLayersV2
            className={cx(
              "weg-item-preview-container",
              $settings.value.position.toLowerCase(),
            )}
            onMouseMoveCapture={(e) => e.stopPropagation()}
            onContextMenu={(e) => {
              e.stopPropagation();
              e.preventDefault();
            }}
            prefix="preview"
          >
            <div className="weg-item-preview-scrollbar">
              {item.windows.map((window) => (
                <UserApplicationPreview
                  key={window.handle}
                  title={window.title}
                  hwnd={window.handle}
                  isFocused={focusedApp?.hwnd === window.handle}
                />
              ))}
              {item.windows.length === 0 && <div className="weg-item-display-name">{item.displayName}</div>}
            </div>
          </BackgroundByLayersV2>
        }
      >
        {itemNode}
      </AnimatedPopover>
    </WithContextMenu>
  );
});
