import { SeelenCommand } from "@seelen-ui/lib";
import { Icon, MissingIcon } from "libs/ui/react/components/Icon/index.tsx";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

import { $delayedFocused, $previews } from "../../shared/state/mod.ts";
import type { TargetedMouseEvent } from "preact";

interface PreviewProps {
  title: string;
  hwnd: number;
}

export const UserApplicationPreview = ({ title, hwnd }: PreviewProps) => {
  const preview = $previews.value[hwnd];

  function onClick() {
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd,
      wasFocused: $delayedFocused.value?.hwnd === hwnd,
    });
  }

  function onAuxClick(e: TargetedMouseEvent<HTMLDivElement>) {
    if (e.button === 1) {
      invoke(SeelenCommand.WegCloseApp, { hwnd });
    }
  }

  function onClose(e: TargetedMouseEvent<HTMLDivElement>) {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  return (
    <div className="weg-item-preview" onClick={onClick} onAuxClick={onAuxClick}>
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
          : <MissingIcon className="weg-item-no-preview" />}
      </div>
    </div>
  );
};
