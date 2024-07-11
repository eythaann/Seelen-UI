import { Icon } from '../../../../shared/components/Icon';
import { SavedMediaItem } from '../../../../shared/schemas/SeelenWegItems';
import { DraggableItem } from './DraggableItem';
import { convertFileSrc } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Button } from 'antd';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { calcLuminance } from '../../../../toolbar/modules/media/application';
import { Selectors } from '../../shared/store/app';

import './MediaSession.css';

const MAX_LUMINANCE = 210;
const MIN_LUMINANCE = 40;
const BRIGHTNESS_MULTIPLIER = 1.25; // used in css

export function MediaSession({ item }: { item: SavedMediaItem }) {
  const [luminance, setLuminance] = useState(0);

  const session = useSelector(Selectors.mediaLastPlayedSession);

  let src = convertFileSrc(
    session?.thumbnail ? session.thumbnail : LAZY_CONSTANTS.DEFAULT_THUMBNAIL,
  );

  useEffect(() => {
    calcLuminance(src).then(setLuminance).catch(console.error);
  }, [src]);

  useEffect(() => {
    emit('register-media-events');
  }, []);

  const filteredLuminance = Math.max(
    Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE),
    MIN_LUMINANCE,
  );
  const color = filteredLuminance < 125 ? '#efefef' : '#222222';
  return (
    <DraggableItem item={item}>
      <div className="media-session-container">
        <div
          className="media-session"
          style={{
            backgroundColor: `rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`,
          }}
        >
          <img className="media-session-thumbnail" src={src} draggable={false} />
          <img className="media-session-blurred-thumbnail" src={src} draggable={false} />

          <div className="media-session-info">
            <span className="media-session-title" style={{ color }}>
              {session?.title || 'No Media'}
            </span>
            <div className="media-session-actions">
              <Button type="text" size="small">
                <Icon iconName="TbPlayerSkipBackFilled" propsIcon={{ color, size: 12 }} />
              </Button>
              <Button type="text" size="small">
                <Icon
                  iconName={session?.playing ? 'TbPlayerPauseFilled' : 'TbPlayerPlayFilled'}
                  propsIcon={{ color, size: 12 }}
                />
              </Button>
              <Button type="text" size="small">
                <Icon iconName="TbPlayerSkipForwardFilled" propsIcon={{ color, size: 12 }} />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </DraggableItem>
  );
}
