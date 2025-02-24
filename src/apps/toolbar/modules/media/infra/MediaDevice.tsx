import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Button, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';

import { Icon } from 'src/apps/shared/components/Icon';
import { OverflowTooltip } from 'src/apps/shared/components/OverflowTooltip';

import { MediaDevice } from '../../shared/store/domain';

export function Device({ device }: { device: MediaDevice }) {
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
    </div>
  );
}

export function DeviceGroup({ devices }: { devices: MediaDevice[] }) {
  return (
    <div className="media-device-group">
      {devices.map((d) => (
        <Device key={d.id} device={d} />
      ))}
    </div>
  );
}