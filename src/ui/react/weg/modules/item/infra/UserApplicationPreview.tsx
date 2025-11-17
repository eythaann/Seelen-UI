import { SeelenCommand } from "@seelen-ui/lib";
import { Icon } from "@shared/components/Icon";
import { cx } from "@shared/styles";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { tempDir } from "@tauri-apps/api/path";
import { Spin } from "antd";
import { type MouseEvent, useEffect, useReducer, useState } from "react";

import type { HWND } from "../../shared/store/domain.ts";

import { $settings } from "../../shared/state/mod.ts";
interface PreviewProps {
  title: string;
  hwnd: HWND;
  isFocused: boolean;
}

const TEMP_FOLDER = await tempDir();

export const UserApplicationPreview = ({ title, hwnd, isFocused }: PreviewProps) => {
  const imageUrl = convertFileSrc(`${TEMP_FOLDER}${hwnd}.png`);

  const [imageSrc, setImageSrc] = useState<string | null>(imageUrl);
  const [_, forceUpdate] = useReducer((x) => x + 1, 0);

  useEffect(() => {
    const unlisten = listen(`weg-preview-update-${hwnd}`, () => {
      setImageSrc(imageUrl);
      forceUpdate(_);
    });
    return () => {
      unlisten.then((unlisten) => unlisten()).catch(console.error);
    };
  }, []);

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
          wasFocused: isFocused,
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
          {imageSrc
            ? (
              <img
                className="weg-item-preview-image"
                src={imageSrc + `?${new Date().getTime()}`}
                onError={() => setImageSrc(null)}
              />
            )
            : <Spin className="weg-item-preview-spin" />}
        </div>
      )}
    </div>
  );
};
