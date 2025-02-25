import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Button, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';

import { Icon } from 'src/apps/shared/components/Icon';
import { OverflowTooltip } from 'src/apps/shared/components/OverflowTooltip';

import { MediaDevice } from '../../shared/store/domain';

export function Device({
  device,
  setViewDeviceId,
}: {
  device: MediaDevice;
  setViewDeviceId: (id: string) => void;
}) {
  const { t } = useTranslation();

  const onClickMultimedia = () => {
    if (!device.isDefaultMultimedia) {
      invoke(SeelenCommand.MediaSetDefaultDevice, { id: device.id, role: 'multimedia' })
        .then(() => invoke(SeelenCommand.MediaSetDefaultDevice, { id: device.id, role: 'console' }))
        .catch(console.error);
    }
  };

  const onClickCommunications = () => {
    if (!device.isDefaultCommunications) {
      invoke(SeelenCommand.MediaSetDefaultDevice, { id: device.id, role: 'communications' }).catch(
        console.error,
      );
    }
  };

  return (
    <div className="media-device">
      <Button.Group size="small" style={{ width: 50 }}>
        <Tooltip title={t('media.device.multimedia')}>
          <Button
            type={device.isDefaultMultimedia ? 'primary' : 'default'}
            onClick={onClickMultimedia}
          >
            <Icon iconName="IoMusicalNotes" size={18} />
          </Button>
        </Tooltip>
        <Tooltip title={t('media.device.comunications')}>
          <Button
            type={device.isDefaultCommunications ? 'primary' : 'default'}
            onClick={onClickCommunications}
          >
            <Icon iconName="FaPhoneFlip" />
          </Button>
        </Tooltip>
      </Button.Group>
      <OverflowTooltip text={device.name} />
      <button className="media-device-action" onClick={() => setViewDeviceId(device.id)}>
        <Icon iconName="RiEqualizerLine" />
      </button>
    </div>
  );
}

export function DeviceGroup({
  devices,
  setViewDeviceId,
}: {
  devices: MediaDevice[];
  setViewDeviceId: (id: string) => void;
}) {
  return (
    <div className="media-device-group">
      {devices.map((d) => (
        <Device key={d.id} device={d} setViewDeviceId={setViewDeviceId} />
      ))}
    </div>
  );
}
