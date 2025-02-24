import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';
import { Icon } from 'src/apps/shared/components/Icon';

import { DeviceGroup } from './MediaDevice';
import { MediaPlayerSession } from './MediaPlayer';
import { VolumeControl } from './VolumeControl';

interface Props {
  setViewDeviceId: (id: string) => void;
}

export function MediaMainView({ setViewDeviceId }: Props) {
  const { t } = useTranslation();

  const inputs = useSelector(Selectors.mediaInputs);
  const defaultInput = inputs.find((d) => d.isDefaultMultimedia);

  const outputs = useSelector(Selectors.mediaOutputs);
  const defaultOutput = outputs.find((d) => d.isDefaultMultimedia);

  const sessions = useSelector(Selectors.mediaSessions);

  return (
    <>
      <span className="media-control-label">{t('media.master_volume')}</span>
      {!!defaultOutput && (
        <VolumeControl
          value={defaultOutput.volume}
          deviceId={defaultOutput.id}
          icon={
            <Icon iconName={defaultOutput.muted ? 'IoVolumeMuteOutline' : 'IoVolumeHighOutline'} />
          }
          onRightAction={() => setViewDeviceId(defaultOutput.id)}
        />
      )}

      {!!defaultInput && (
        <VolumeControl
          value={defaultInput.volume}
          deviceId={defaultInput.id}
          icon={<Icon iconName={defaultInput.muted ? 'BiMicrophoneOff' : 'BiMicrophone'} />}
          onRightAction={() => setViewDeviceId(defaultInput.id)}
        />
      )}

      {outputs.length > 0 && (
        <>
          <span className="media-control-label">{t('media.output_device')}</span>
          <DeviceGroup devices={outputs} />
        </>
      )}

      {inputs.length > 0 && (
        <>
          <span className="media-control-label">{t('media.input_device')}</span>
          <DeviceGroup devices={inputs} />
        </>
      )}

      {sessions.length > 0 && (
        <>
          <span className="media-control-label">{t('media.players')}</span>
          <div className="media-control-session-list">
            {sessions.map((session, index) => (
              <MediaPlayerSession key={index} session={session} />
            ))}
          </div>
        </>
      )}
    </>
  );
}
