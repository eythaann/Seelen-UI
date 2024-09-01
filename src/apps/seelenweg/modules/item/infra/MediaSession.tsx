import { Icon } from '../../../../shared/components/Icon';
import { SavedMediaItem } from '../../../../shared/schemas/SeelenWegItems';
import { WithContextMenu } from '../../../components/WithContextMenu';
import { DraggableItem } from './DraggableItem';
import { getMenuForItem } from './Menu';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Button } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { calcLuminance } from '../../../../toolbar/modules/media/application';
import { Selectors } from '../../shared/store/app';

import './MediaSession.css';

const MAX_LUMINANCE = 210;
const MIN_LUMINANCE = 40;
const BRIGHTNESS_MULTIPLIER = 1.5; // used in css

export function MediaSession({ item }: { item: SavedMediaItem }) {
  const [luminance, setLuminance] = useState(0);

  const sessions = useSelector(Selectors.mediaSessions);
  const session = sessions.find((s) => s.default);

  let src = convertFileSrc(
    session?.thumbnail ? session.thumbnail : LAZY_CONSTANTS.DEFAULT_THUMBNAIL,
  );

  const { t } = useTranslation();

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

  const onClickBtn = (cmd: string) => {
    if (session) {
      invoke(cmd, { id: session.id }).catch(console.error);
    }
  };

  return (
    <DraggableItem item={item}>
      <WithContextMenu items={getMenuForItem(t, item)}>
        <div className="media-session-container" onContextMenu={(e) => e.stopPropagation()}>
          <div
            className="media-session"
            style={{
              backgroundColor: `rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`,
            }}
          >
            <div className="media-session-thumbnail-container">
              {session?.owner && (
                <img
                  className="media-session-app-icon"
                  src={convertFileSrc(
                    session.owner.iconPath
                      ? session.owner.iconPath
                      : LAZY_CONSTANTS.MISSING_ICON_PATH,
                  )}
                  draggable={false}
                />
              )}
              <img className="media-session-thumbnail" src={src} draggable={false} />
            </div>
            <img className="media-session-blurred-thumbnail" src={src} draggable={false} />

            <div className="media-session-info">
              <span className="media-session-title" style={{ color }}>
                {session?.title || 'No Media'}
              </span>
              <div className="media-session-actions">
                <Button type="text" size="small" onClick={onClickBtn.bind(null, 'media_prev')}>
                  <Icon iconName="TbPlayerSkipBackFilled" propsIcon={{ color, size: 12 }} />
                </Button>
                <Button
                  type="text"
                  size="small"
                  onClick={onClickBtn.bind(null, 'media_toggle_play_pause')}
                >
                  <Icon
                    iconName={session?.playing ? 'TbPlayerPauseFilled' : 'TbPlayerPlayFilled'}
                    propsIcon={{ color, size: 12 }}
                  />
                </Button>
                <Button type="text" size="small" onClick={onClickBtn.bind(null, 'media_next')}>
                  <Icon iconName="TbPlayerSkipForwardFilled" propsIcon={{ color, size: 12 }} />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </WithContextMenu>
    </DraggableItem>
  );
}
