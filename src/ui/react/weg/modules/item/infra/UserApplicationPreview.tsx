import { SeelenCommand } from "@seelen-ui/lib";
import { Icon, MissingIcon } from "libs/ui/react/components/Icon/index.tsx";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { MouseEvent } from "react";

import type { HWND } from "../../shared/types.ts";

import { $delayedFocused, $previews } from "../../shared/state/mod.ts";
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
      className="weg-item-preview"
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
    </div>
  );
};
