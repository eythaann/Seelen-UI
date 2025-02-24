import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Select } from 'antd';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { FileIcon, Icon } from 'src/apps/shared/components/Icon';

import { RootState } from '../../shared/store/domain';

import { VolumeControl } from './VolumeControl';

interface Props {
  onBack: () => void;
  deviceId: string;
}

export function MediaMixerView({ onBack, deviceId }: Props) {
  const { t } = useTranslation();

  const device = useSelector((state: RootState) => {
    return (
      state.mediaInputs.find((d) => d.id === deviceId) ||
      state.mediaOutputs.find((d) => d.id === deviceId)
    );
  });

  const channels = device?.sessions.toSorted((a, b) => a.name.localeCompare(b.name));

  return (
    <>
      <div className="media-device-header">
        <button className="media-device-back" onClick={onBack}>
          <Icon iconName="IoArrowBack" />
        </button>
        <span className="media-device-title">{device?.name}</span>
      </div>

      <span className="media-control-label">{t('media.device.spacial')}</span>
      <Select style={{ width: '100%' }} />

      <span className="media-control-label">{t('media.device.mixer')}</span>
      <div className="media-device-mixer">
        {channels?.map((channel) => (
          <VolumeControl
            key={channel.id}
            value={channel.volume}
            deviceId={deviceId}
            sessionName={channel.isSystem ? t('media.device.channel.system') : channel.name}
            sessionId={channel.id}
            icon={
              <div className="media-device-mixer-entry-icon">
                {channel.isSystem ? (
                  <Icon iconName="BsSpeaker" size={24} />
                ) : (
                  <FileIcon path={channel.iconPath} style={{ height: '100%' }} />
                )}
                <Icon iconName={channel.muted ? 'IoVolumeMuteOutline' : 'IoVolumeHighOutline'} />
              </div>
            }
          />
        ))}
      </div>

      <div className="media-device-footer">
        <button
          className="media-device-footer-button"
          onClick={() => {
            console.debug(`ms-settings:sound-properties?endpointId=${deviceId}`);
            invoke(SeelenCommand.OpenFile, {
              path: `ms-settings:sound-properties?endpointId=${deviceId}`,
            });
          }}
        >
          {t('media.device.settings')}
        </button>
      </div>
    </>
  );
}
