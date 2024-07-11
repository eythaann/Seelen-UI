import { Icon } from '../../../../shared/components/Icon';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Button, Popover, Slider } from 'antd';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgrounByLayers/infra';
import { useAppBlur } from '../../shared/hooks/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import { calcLuminance } from '../application';

import { MediaSession } from '../../shared/store/domain';

import './index.css';

const MAX_LUMINANCE = 210;
const MIN_LUMINANCE = 40;
const BRIGHTNESS_MULTIPLIER = 1.25; // used in css

function MediaSession({ session }: { session: MediaSession }) {
  const [luminance, setLuminance] = useState(0);

  let src = convertFileSrc(
    session.thumbnail ? session.thumbnail : LAZY_CONSTANTS.DEFAULT_THUMBNAIL,
  );

  useEffect(() => {
    calcLuminance(src).then(setLuminance).catch(console.error);
  }, [src]);

  const filteredLuminance = Math.max(Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE), MIN_LUMINANCE);
  const color = filteredLuminance < 125 ? '#efefef' : '#222222';

  return (
    <div
      className="media-session"
      style={{
        backgroundColor: `rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`,
      }}
    >
      <img className="media-session-thumbnail" src={src} draggable={false} />
      <img className="media-session-blurred-thumbnail" src={src} draggable={false} />

      <div className="media-session-info" style={{ color }}>
        <h4 className="media-session-title">{session.title}</h4>
        <span className="media-session-author">{session.author}</span>
        <div className="media-session-actions">
          <Button type="text">
            <Icon iconName="TbPlayerSkipBackFilled" propsIcon={{ color }} />
          </Button>
          <Button type="text">
            <Icon
              iconName={session.playing ? 'TbPlayerPauseFilled' : 'TbPlayerPlayFilled'}
              propsIcon={{ color }}
            />
          </Button>
          <Button type="text">
            <Icon iconName="TbPlayerSkipForwardFilled" propsIcon={{ color }} />
          </Button>
        </div>
      </div>
    </div>
  );
}

export function WithMediaControls({ children }: PropsWithChildren) {
  const [openPreview, setOpenPreview] = useState(false);

  const volume = useSelector(Selectors.mediaVolume);
  const sessions = useSelector(Selectors.mediaSessions);

  useAppBlur(() => {
    setOpenPreview(false);
  });

  const onChangeVolume = (value: number) => {
    invoke('set_volume_level', { level: value });
  };

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <BackgroundByLayersV2 className="media-control" amount={1}>
          <div className="media-control-volume">
            <Button type="text" onClick={() => invoke('media_toggle_mute')}>
              <Icon iconName="IoVolumeHighOutline" />
            </Button>
            <Slider
              value={volume}
              onChange={onChangeVolume}
              min={0}
              max={1}
              step={0.01}
              tooltip={{
                formatter: (value) => `${(100 * (value || 0)).toFixed(0)}`,
              }}
            />
            <Button type="text" onClick={() => invoke('open_file', { path: 'ms-settings:sound' })}>
              <Icon iconName="RiEqualizerLine" />
            </Button>
          </div>
          {sessions.length > 0 && (
            <div className="media-control-session-list">
              {sessions.map((session, index) => (
                <MediaSession key={index} session={session} />
              ))}
            </div>
          )}
        </BackgroundByLayersV2>
      }
    >
      <div>{children}</div>
    </Popover>
  );
}
