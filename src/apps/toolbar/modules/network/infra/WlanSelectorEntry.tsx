import { IconName } from '@icons';
import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Button, Input, Tooltip } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { WlanBssEntry, WlanProfile } from '../domain';

import { Icon } from '../../../../shared/components/Icon';
import { cx } from '../../../../shared/styles';

export function WlanSelectorEntry(props: {
  group: [WlanBssEntry, ...WlanBssEntry[]];
  selected: boolean;
  onClick?: () => void;
}) {
  let { group, selected, onClick } = props;
  let entry = group[0];
  let isHiddenGroup = !entry.ssid;

  let [loading, setLoading] = useState(false);
  let [showFields, setShowFields] = useState(false);
  let [showErrors, setShowErrors] = useState(false);

  let [ssid, setSsid] = useState(entry.ssid || '');
  let [password, setPassword] = useState('');

  const { t } = useTranslation();

  useEffect(() => {
    setShowFields(selected && !entry.known && (!entry.ssid || entry.secured));
    setSsid(entry.ssid || '');
    setPassword('');
    setShowErrors(false);
  }, [selected]);

  function onConnection() {
    function onfulfilled(success: boolean) {
      setLoading(false);
      setShowFields(!success);
      setShowErrors(!success);
    }

    function onrejected(error: any) {
      console.error(error);
      setLoading(false);
      setShowErrors(true);
    }

    setLoading(true);

    if (entry.connected) {
      invoke(SeelenCommand.WlanDisconnect).then(() => setLoading(false), onrejected);
      return;
    }

    if (showFields) {
      invoke<boolean>('wlan_connect', { ssid, password, hidden: !entry.ssid }).then(
        onfulfilled,
        onrejected,
      );
      return;
    }

    invoke<WlanProfile[]>('wlan_get_profiles')
      .then((profiles) => {
        let profile = profiles.find((profile) => profile.ssid === entry.ssid);
        if (!profile) {
          setShowFields(true);
          setLoading(false);
          return;
        }

        invoke<boolean>('wlan_connect', {
          ssid: profile.ssid,
          password: profile.password,
          hidden: !entry.ssid,
        }).then(onfulfilled, onrejected);
      })
      .catch(onrejected);
  }

  let signalIcon: IconName = 'GrWifiNone';
  if (entry.signal > 75) {
    signalIcon = 'GrWifi';
  } else if (entry.signal > 50) {
    signalIcon = 'GrWifiMedium';
  } else if (entry.signal > 25) {
    signalIcon = 'GrWifiLow';
  }

  let frequencies: string[] = [];
  if (group.some((e) => e.channelFrequency < 5_000_000)) {
    frequencies.push('2.4G');
  }
  if (group.some((e) => e.channelFrequency > 5_000_000)) {
    frequencies.push('5G');
  }

  return (
    <div
      key={entry.bssid}
      className={cx('wlan-entry', {
        'wlan-entry-selected': selected,
      })}
      onClick={onClick}
    >
      <div className="wlan-entry-info">
        <Icon iconName={signalIcon} size={20} />
        <span className="wlan-entry-info-ssid">{entry.ssid || `${t('network.hidden')} (${group.length})`}</span>
        {!isHiddenGroup && <div className="wlan-entry-info-channel">{frequencies.join('/')}</div>}
        {!isHiddenGroup && entry.secured && (
          <Tooltip title={t('network.secured')}>
            <Icon iconName="PiPasswordFill" />
          </Tooltip>
        )}
      </div>
      {showFields && (
        <form className="wlan-entry-fields">
          {!entry.ssid && (
            <Input
              type="text"
              placeholder="SSID"
              value={ssid}
              status={showErrors ? 'error' : undefined}
              onChange={(e) => setSsid(e.target.value)}
              autoFocus
              onPressEnter={(e) => (e.currentTarget.nextSibling as HTMLInputElement)?.focus()}
            />
          )}
          <Input
            type="password"
            placeholder={t('network.placeholder.password')}
            value={password}
            status={showErrors ? 'error' : undefined}
            onChange={(e) => setPassword(e.target.value)}
            onPressEnter={onConnection}
            autoFocus={!!entry.ssid}
          />
        </form>
      )}
      {selected && (
        <div className="wlan-entry-actions">
          <Button
            type={entry.connected ? 'default' : 'primary'}
            onClick={onConnection}
            loading={loading}
            disabled={loading}
          >
            {entry.connected ? t('network.disconnect') : t('network.connect')}
          </Button>
        </div>
      )}
    </div>
  );
}
