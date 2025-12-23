import { SeelenCommand } from "@seelen-ui/lib";
import type { MediaPlayer } from "@seelen-ui/lib/types";
import { FileIcon, Icon } from "libs/ui/react/components/Icon/index.tsx";
import { path } from "@tauri-apps/api";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { Button, Tooltip } from "antd";
import { useEffect, useState } from "react";

import { calcLuminance } from "../application.ts";

const MAX_LUMINANCE = 210;
const MIN_LUMINANCE = 40;
const BRIGHTNESS_MULTIPLIER = 1.5; // used in css

const DEFAULT_THUMBNAIL = await path.resolve(
  await path.resourceDir(),
  "static",
  "icons",
  "music_thumbnail.jpg",
);

export function MediaPlayerSession({ session }: { session: MediaPlayer }) {
  const [luminance, setLuminance] = useState(0);

  let thumbnailSrc = convertFileSrc(
    session?.thumbnail ? session.thumbnail : DEFAULT_THUMBNAIL,
  );

  useEffect(() => {
    calcLuminance(thumbnailSrc).then(setLuminance).catch(console.error);
  }, [thumbnailSrc]);

  const filteredLuminance = Math.max(
    Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE),
    MIN_LUMINANCE,
  );
  const color = filteredLuminance < 125 ? "#efefef" : "#222222";

  const onClickBtn = (cmd: string) => {
    invoke(cmd, { id: session.umid }).catch(console.error);
  };

  return (
    <div
      className="media-session"
      style={{
        backgroundColor: `rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`,
      }}
    >
      <img className="media-session-blurred-thumbnail" src={thumbnailSrc} />
      <div className="media-session-thumbnail-container">
        <Tooltip title={session.owner.name} placement="bottom">
          <FileIcon className="media-session-app-icon" umid={session.umid} />
        </Tooltip>
        <img className="media-session-thumbnail" src={thumbnailSrc} />
      </div>

      <div className="media-session-info" style={{ color }}>
        <h4 className="media-session-title">{session.title}</h4>
        <span className="media-session-author">{session.author}</span>
        <div className="media-session-actions">
          <Button
            type="text"
            onClick={onClickBtn.bind(null, SeelenCommand.MediaPrev)}
          >
            <Icon iconName="TbPlayerSkipBackFilled" color={color} />
          </Button>
          <Button
            type="text"
            onClick={onClickBtn.bind(null, SeelenCommand.MediaTogglePlayPause)}
          >
            <Icon
              iconName={session.playing ? "TbPlayerPauseFilled" : "TbPlayerPlayFilled"}
              color={color}
            />
          </Button>
          <Button
            type="text"
            onClick={onClickBtn.bind(null, SeelenCommand.MediaNext)}
          >
            <Icon iconName="TbPlayerSkipForwardFilled" color={color} />
          </Button>
        </div>
      </div>
    </div>
  );
}
