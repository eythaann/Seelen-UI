import { SeelenCommand } from '@seelen-ui/lib';
import { FileIcon, Icon } from '@shared/components/Icon';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

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

  if (!device) {
    return (
      <div className="media-device-header">
        <button className="media-device-back" onClick={onBack}>
          <Icon iconName="IoArrowBack" />
        </button>
        <span className="media-device-title">{t('media.device.missing')}</span>
      </div>
    );
  }

  const sessionsIds = new Set();
  const sessions = device.sessions
    .toSorted((a, b) => a.name.localeCompare(b.name))
    .filter((s) => {
      let previusContains = sessionsIds.has(s.id);
      sessionsIds.add(s.id);
      return !previusContains;
    });
  const iconName = device.type === 'input' ? 'BiMicrophone' : 'IoVolumeHighOutline';
  const mutedIconName = device.type === 'input' ? 'BiMicrophoneOff' : 'IoVolumeMuteOutline';

  return (
    <>
      <div className="media-device-header">
        <button className="media-device-back" onClick={onBack}>
          <Icon iconName="IoArrowBack" />
        </button>
        <span className="media-device-title">{device.name}</span>
      </div>

      <span className="media-control-label">{t('media.device.volume')}</span>
      <VolumeControl
        deviceId={deviceId}
        value={device.volume}
        icon={<Icon iconName={device.muted ? mutedIconName : iconName} />}
      />

      {/*
      Maybe one day this can be implemented but for now I give up
      <span className="media-control-label">{t('media.device.spacial')}</span>
      <Select style={{ width: '100%' }} />
      */}

      {device.type != 'input' && (
        <>
          <span className="media-control-label">{t('media.device.mixer')}</span>
          <div className="media-device-mixer">
            {sessions.map((channel) => (
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
                    <Icon iconName={channel.muted ? mutedIconName : iconName} />
                  </div>
                }
              />
            ))}
          </div>
        </>
      )}

      <div className="media-device-footer">
        <button
          className="media-device-footer-button"
          onClick={() => {
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
