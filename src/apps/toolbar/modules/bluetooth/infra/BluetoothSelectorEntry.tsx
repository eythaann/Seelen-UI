import { BluetoothDevices } from '@seelen-ui/lib';
import { BluetoothDevice, BluetoothDevicePairShowPinRequest } from '@seelen-ui/lib/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import { UnlistenFn } from '@tauri-apps/api/event';
import { Button, Input, Tooltip } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Icon, MissingIcon } from '../../../../shared/components/Icon';
import { cx } from '../../../../shared/styles';

export function BluetoothSelectorEntry(props: {
  device: BluetoothDevice;
  selected: boolean;
  onClick?: () => void;
}) {
  let { device, selected, onClick } = props;

  let [loading, setLoading] = useState(false);
  let [showFields, setShowFields] = useState(false);
  let [passphrase, setPassphrase] = useState('');
  let [showErrors, setShowErrors] = useState(false);
  let [confirmationPhase, setConfirmationPhase] = useState(false);
  let [unsubscribtion, setUnsubscribtion] = useState<UnlistenFn | undefined>(undefined);

  const { t } = useTranslation();

  useEffect(() => {
    setShowFields(selected && confirmationPhase);
    setShowErrors(false);
    setPassphrase('');
    setLoading(false);
    if (unsubscribtion !== undefined) {
      unsubscribtion();
      setUnsubscribtion(undefined);
    }
    setConfirmationPhase(false);
  }, [selected]);

  const onAction = async (device: BluetoothDevice, accept?: boolean) => {
    setLoading(true);
    if (confirmationPhase && accept !== undefined) {
      try {
        await BluetoothDevices.confirmPair(accept, passphrase);
      } catch {
        setShowErrors(true);
      } finally {
        setLoading(false);
        setConfirmationPhase(false);
      }
    } else {
      if (device.paired) {
        try {
          await BluetoothDevices.forgetDevice(device.id);
        } catch {
          setShowErrors(true);
        } finally {
          setLoading(false);
        }
      } else {
        try {
          setUnsubscribtion(await BluetoothDevices.onPairRequest(onPairRequest));
          //TODO(Eythaan): from here I can not test the process. It should send this event to the UI, but it not arrives!
          await BluetoothDevices.pairDevice(device.address);
        } catch {
          setShowErrors(true);
        } finally {
          setLoading(false);
        }
      }
    }
  };

  const onPairRequest = (param) => {
    if (unsubscribtion) {
      unsubscribtion();
    }
    setUnsubscribtion(undefined);

    if (param.pin !== undefined) {
      setPassphrase(param.pin);
      setShowFields(true);
      setConfirmationPhase(param.confirmationNeeded);
    }

    setLoading(false);
  };

  // TODO(Eythaan): the style left the copy from wlan so you can modify it to your desire!
  // TODO(Eythaan): Also the confirmation process did not caried to the end anytime - just teoretically works. Please note that there are 3 run through possible!
  // !!! DevicePairingKinds::ConfirmPinMatch (both side see same and click ok) | DevicePairingKinds::ProvidePin (one side gives pin for auth other side confirm, so the item will disappear after pair success) | DevicePairingKinds::DisplayPin (other side confirm) !!!
  return (
    <div
      key={device.id}
      className={cx('wlan-entry', {
        'wlan-entry-selected': selected,
      })}
      onClick={onClick}
    >
      <div className="wlan-entry-info">
        <Icon iconName={device.connected ? 'TbBluetoothConnected' : 'TbBluetooth' } size={20} />
        <span className="wlan-entry-info-ssid">{device.name}</span>
        <Tooltip title={`${device.majorClass} \\ ${device.minorSubClass == 'Uncategorized' ? device.minorMainClass : `${device.minorMainClass}, ${device.minorSubClass}`}`}>
          {device.iconPath ? (
            <img className="bluetooth-entry-info-img" src={convertFileSrc(device.iconPath)} />
          ) : (
            <MissingIcon />
          )}
        </Tooltip>
        { device.isBluetoothLoweenergy &&
          <Tooltip title={t('bluetooth.lowenergy')}>
            <Icon iconName="MdOutlineEnergySavingsLeaf" size={20} />
          </Tooltip>
        }
      </div>
      {showFields && (
        <form className="wlan-entry-fields">
          <Input
            type="text"
            placeholder={t('bluetooth.placeholder.passphrase')}
            value={passphrase}
            status={showErrors ? 'error' : undefined}
            onChange={(e) => setPassphrase(e.target.value)}
            onPressEnter={() => onAction(device, confirmationPhase ? true : undefined)}
            autoFocus={showFields}
          />
        </form>
      )}
      {selected && (
        <div className="wlan-entry-actions">
          <Button
            type={device.paired ? 'default' : 'primary'}
            onClick={() => onAction(device, confirmationPhase ? true : undefined)}
            loading={loading}
            disabled={loading}
          >
            {device.paired ? t('bluetooth.forget') : t('bluetooth.pair')}
          </Button>
          { confirmationPhase &&
            <Button
              type="default"
              onClick={() => onAction(device, false)}
              loading={loading}
              disabled={loading}
            >
              {t('bluetooth.cancel')}
            </Button>
          }
        </div>
      )}
    </div>
  );
}
