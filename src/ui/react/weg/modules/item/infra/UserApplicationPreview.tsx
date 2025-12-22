import { SeelenCommand } from "@seelen-ui/lib";
import { Icon, MissingIcon } from "@shared/components/Icon";
import { cx } from "@shared/styles";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { MouseEvent } from "react";

import type { HWND } from "../../shared/types.ts";

import { $delayedFocused, $previews, $settings } from "../../shared/state/mod.ts";
interface PreviewProps {
  title: string;
  hwnd: HWND;
}

export const UserApplicationPreview = ({ title, hwnd }: PreviewProps) => {
  const preview = $previews.value[hwnd];

  const onClose = (e: MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  };

  return (
    <div
      className={cx("weg-item-preview", {
        "weg-item-preview-thumbnail-disabled": !$settings.value.thumbnailGenerationEnabled,
      })}
      onClick={() => {
        invoke(SeelenCommand.WegToggleWindowState, {
          hwnd,
          wasFocused: $delayedFocused.value?.hwnd === hwnd,
        });
      }}
      onAuxClick={(e) => {
        if (e.button === 1) {
          invoke(SeelenCommand.WegCloseApp, { hwnd });
        }
      }}
    >
      <div className="weg-item-preview-topbar">
        <div className="weg-item-preview-title">{title}</div>
        <div className="weg-item-preview-close" onClick={onClose}>
          <Icon iconName="IoClose" />
        </div>
      </div>
      {$settings.value.thumbnailGenerationEnabled && (
        <div className="weg-item-preview-image-container">
          {preview
            ? (
              <img
                className="weg-item-preview-image"
                src={convertFileSrc(preview.path) + "?v=" + preview.hash}
              />
            )
            : <MissingIcon />}
        </div>
      )}
    </div>
  );
};
