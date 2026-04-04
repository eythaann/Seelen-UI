import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { SeelenWegSide, type UserAppWindow } from "@seelen-ui/lib/types";
import { FileIcon } from "libs/ui/react/components/Icon/index.tsx";
import { useWindowFocusChange } from "libs/ui/react/utils/hooks.ts";
import { cx } from "libs/ui/react/utils/styling.ts";
import moment from "moment";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import type { AppOrFileWegItem } from "../../shared/types.ts";

import {
  $delayedFocused,
  $focused,
  $interactables,
  $notifications,
  $open_popups,
  $settings,
} from "../../shared/state/mod.ts";
import { getDockContextMenuAlignment } from "../../shared/state/settings.ts";
import { getWindowsForItem } from "../../shared/state/windows.ts";
import { getUserApplicationContextMenu, launchItem } from "./UserApplicationContextMenu.tsx";
import { UserApplicationPreview } from "./UserApplicationPreview.tsx";
import { Flex, Popover } from "antd";

interface Props {
  item: AppOrFileWegItem;
  isOverlay?: boolean;
}

interface InnerProps extends Props {
  windows: UserAppWindow[];
}

function UserApplicationItem({ item, isOverlay, windows }: InnerProps) {
  const [openPreview, setOpenPreview] = useState(false);
  const [blockUntil, setBlockUntil] = useState(moment(new Date()));

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
    }
  });

  const onContextMenu = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      setOpenPreview(false);
      const { alignX, alignY } = getDockContextMenuAlignment($settings.value.position);
      invoke(SeelenCommand.TriggerContextMenu, {
        menu: { ...getUserApplicationContextMenu(t, item, windows), alignX, alignY },
        forwardTo: null,
      });
    },
    [item, windows, t],
  );

  const notificationsCount = $notifications.value.filter((n) => n.appUmid === item.umid).length;
  const itemLabel = $settings.value.showWindowTitle && windows.length ? windows[0]!.title : null;

  const itemNode = (
    <div
      className="weg-item"
      onClick={() => {
        const window = windows[0];
        if (!window) {
          launchItem(item, false);
        } else {
          invoke(SeelenCommand.WegToggleWindowState, {
            hwnd: window.hwnd,
            wasFocused: $delayedFocused.value?.hwnd === window.hwnd,
          });
        }
      }}
      onAuxClick={(e) => {
        const window = windows[0];
        if (e.button === 1 && window) {
          invoke(SeelenCommand.WegCloseApp, { hwnd: window.hwnd });
        }
      }}
      onContextMenu={onContextMenu}
    >
      <FileIcon
        className="weg-item-icon"
        path={item.relaunch?.icon || item.path}
        umid={item.umid}
      />
      {itemLabel && <div className="weg-item-title">{itemLabel}</div>}
      {notificationsCount > 0 && <div className="weg-item-notification-badge">{notificationsCount}</div>}
      {$settings.value.showInstanceCounter && windows.length > 1 && (
        <div className="weg-item-instance-counter-badge">{windows.length}</div>
      )}
      {!$settings.value.showWindowTitle && (
        <div
          className={cx("weg-item-open-sign", {
            "weg-item-open-sign-active": windows.length > 0,
            "weg-item-open-sign-focused": windows.some((w) => w.hwnd === $focused.value.hwnd),
          })}
        />
      )}
    </div>
  );

  if (isOverlay) {
    return itemNode;
  }

  return (
    <Popover
      open={openPreview}
      placement={calculatePlacement($settings.value.position)}
      onOpenChange={(open) => {
        setOpenPreview(open && moment(new Date()) > blockUntil);
        $open_popups.value[item.id] = open;
      }}
      trigger="hover"
      arrow={false}
      content={
        <div
          className={cx("weg-item-preview-container", $settings.value.position.toLowerCase())}
          onMouseMoveCapture={(e) => e.stopPropagation()}
          onContextMenu={(e) => {
            e.stopPropagation();
            e.preventDefault();
          }}
          prefix="preview"
        >
          <div className="weg-item-preview-scrollbar">
            {windows.map((window) => (
              <UserApplicationPreview key={window.hwnd} title={window.title} hwnd={window.hwnd} />
            ))}
            {windows.length === 0 && <div className="weg-item-display-name">{item.displayName}</div>}
          </div>
        </div>
      }
    >
      {itemNode}
    </Popover>
  );
}

export function UserApplication({ item, isOverlay }: Props) {
  const windows = getWindowsForItem(item, $interactables.value);

  const { splitWindows, spaceBetweenItems } = $settings.value;
  if (splitWindows && windows.length > 1) {
    return (
      <Flex align="center" gap={spaceBetweenItems}>
        {windows.map((window) => (
          <UserApplicationItem
            key={window.hwnd}
            item={item}
            isOverlay={isOverlay}
            windows={[window]}
          />
        ))}
      </Flex>
    );
  }

  return <UserApplicationItem item={item} isOverlay={isOverlay} windows={windows} />;
}
