import { useComputed } from "@preact/signals";
import { SeelenWegSide } from "@seelen-ui/lib/types";
import { FileIcon, Icon } from "libs/ui/react/components/Icon/index.tsx";
import { cx } from "libs/ui/react/utils/styling.ts";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { resolve, resourceDir } from "@tauri-apps/api/path";
import { Button } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { calcLuminance } from "../application.ts";

import type { MediaWegItem } from "../../shared/types.ts";

import { WithContextMenu } from "../../../components/WithContextMenu.tsx";
import { $players, $settings } from "../../shared/state/mod.ts";
import { getMenuForItem } from "./Menu.tsx";

import "./MediaSession.css";

const MAX_LUMINANCE = 210;
const MIN_LUMINANCE = 40;
const BRIGHTNESS_MULTIPLIER = 1.5; // used in css

const DEFAULT_THUMBNAIL = await resolve(
  await resourceDir(),
  "static",
  "icons",
  "music_thumbnail.jpg",
);

export function MediaSession({ item }: { item: MediaWegItem }) {
  const [luminance, setLuminance] = useState(150);

  const $dock_position = useComputed(() => $settings.value.position);
  const session = $players.value.find((s) => s.default);

  let thumbnailSrc = convertFileSrc(
    session?.thumbnail ? session.thumbnail : DEFAULT_THUMBNAIL,
  );

  const { t } = useTranslation();

  useEffect(() => {
    calcLuminance(thumbnailSrc).then(setLuminance).catch(console.error);
  }, [thumbnailSrc]);

  const filteredLuminance = Math.max(
    Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE),
    MIN_LUMINANCE,
  );
  const color = filteredLuminance < 125 ? "#efefef" : "#222222";

  const onClickBtn = (cmd: string) => {
    if (session) {
      invoke(cmd, { id: session.umid }).catch(console.error);
    }
  };

  const isHorizontal = $dock_position.value === SeelenWegSide.Bottom ||
    $dock_position.value === SeelenWegSide.Top;

  return (
    <WithContextMenu items={getMenuForItem(t, item)}>
      <div
        className={cx("weg-item", "media-session-container", {
          "media-session-container-horizontal": isHorizontal,
          "media-session-container-vertical": !isHorizontal,
        })}
        onContextMenu={(e) => e.stopPropagation()}
        // style={{ zIndex: -1 }} // I don't known why the fuck this item is overlapping but this fixes it
      >
        <div
          className="media-session"
          style={{
            backgroundColor: `rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`,
          }}
        >
          <div className="media-session-blurred-thumbnail-container">
            <img
              className="media-session-blurred-thumbnail"
              src={thumbnailSrc}
              loading="lazy"
            />
          </div>
          <div className="media-session-thumbnail-container">
            <img
              className="media-session-thumbnail"
              src={thumbnailSrc}
              loading="lazy"
            />
            <FileIcon
              className="media-session-app-icon"
              umid={session?.umid}
              noFallback
            />
          </div>
          <div className="media-session-info">
            <span
              className={cx("media-session-title", {
                "media-session-title-default": !session,
              })}
              style={{ color }}
            >
              {session ? session.title : t("media.not_playing")}
            </span>
            {session && (
              <div className="media-session-actions">
                <Button
                  type="text"
                  size="small"
                  onClick={onClickBtn.bind(null, "media_prev")}
                >
                  <Icon
                    iconName="TbPlayerSkipBackFilled"
                    color={color}
                    size={12}
                  />
                </Button>
                <Button
                  type="text"
                  size="small"
                  onClick={onClickBtn.bind(null, "media_toggle_play_pause")}
                >
                  <Icon
                    iconName={session?.playing ? "TbPlayerPauseFilled" : "TbPlayerPlayFilled"}
                    color={color}
                    size={12}
                  />
                </Button>
                <Button
                  type="text"
                  size="small"
                  onClick={onClickBtn.bind(null, "media_next")}
                >
                  <Icon
                    iconName="TbPlayerSkipForwardFilled"
                    color={color}
                    size={12}
                  />
                </Button>
              </div>
            )}
          </div>
        </div>
      </div>
    </WithContextMenu>
  );
}
