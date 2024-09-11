import { invoke } from '@tauri-apps/api/core';
import { Button, Input } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { SeelenCommand } from 'seelen-core';

import { WlanBssEntry, WlanProfile } from '../domain';

import { Icon, IconName } from '../../../../shared/components/Icon';
import { cx } from '../../../../shared/styles';

export function WlanSelectorEntry(props: {
  entry: WlanBssEntry;
  selected: boolean;
  onClick: () => void;
}) {
  let { entry, selected, onClick } = props;

  let [loading, setLoading] = useState(false);
  let [showFields, setShowFields] = useState(false);
  let [showErrors, setShowErrors] = useState(false);

  let [ssid, setSsid] = useState(entry.ssid || '');
  let [password, setPassword] = useState('');

  const { t } = useTranslation();

  useEffect(() => {
    if (!selected) {
      setShowFields(false);
      setShowErrors(false);
      setSsid(entry.ssid || '');
      setPassword('');
    }
  }, [selected]);

  function onConnection() {
    setLoading(true);

    function onrejected(error: any) {
      console.error(error);
      setLoading(false);
      setShowErrors(true);
    }

    if (entry.connected) {
      invoke(SeelenCommand.WlanDisconnect).then(() => setLoading(false), onrejected);
      return;
    }

    if (!entry.ssid && !showFields) {
      setShowFields(true);
      setLoading(false);
      return;
    }

    function onfulfilled(success: boolean) {
      setLoading(false);
      setShowFields(!success);
      setShowErrors(!success);
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

  return (
    <div
      key={entry.bssid}
      className={cx('wlan-entry', {
        'wlan-entry-selected': selected,
      })}
      onClick={onClick}
    >
      <div className="wlan-entry-info">
        <Icon iconName={signalIcon} propsIcon={{ size: 20 }} />
        <span className="wlan-entry-info-ssid">{entry.ssid || t('network.hidden')}</span>
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
          <Button type="primary" onClick={onConnection} loading={loading} disabled={loading}>
            {entry.connected ? t('network.disconnect') : t('network.connect')}
          </Button>
        </div>
      )}
    </div>
  );
}
