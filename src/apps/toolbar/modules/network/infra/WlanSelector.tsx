import { WlanSelectorEntry } from './WlanSelectorEntry';
import { invoke } from '@tauri-apps/api/core';
import { Popover } from 'antd';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../../seelenweg/components/BackgrounByLayers/infra';
import { useAppBlur } from '../../shared/hooks/infra';

import { Selectors } from '../../shared/store/app';

export function WithWlanSelector({ children }: PropsWithChildren) {
  const [openPreview, setOpenPreview] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);

  const entries = useSelector(Selectors.wlanBssEntries);

  const { t } = useTranslation();

  useEffect(() => {
    if (openPreview) {
      invoke('wlan_start_scanning');
    } else {
      setSelected(null);
      invoke('wlan_stop_scanning');
    }
  }, [openPreview]);

  useAppBlur(() => {
    setOpenPreview(false);
  });

  let ssids = new Set<string>();
  let filtered = entries
    .toSorted((a, b) => b.signal - a.signal)
    .filter((entry) => {
      let ssid = entry.ssid || '__HIDDEN_SSID__';
      if (ssids.has(ssid)) {
        return false;
      }
      ssids.add(ssid);
      return true;
    });

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <>
          <div className="wlan-selector">
            <BackgroundByLayers prefix="wlan-selector" layers={1} />
            <div className="wlan-selector-entries">
              {filtered.length === 0 && (
                <div className="wlan-selector-empty">{t('network.not_found')}</div>
              )}
              {filtered.map((entry) => {
                let ssid = entry.ssid || '__HIDDEN_SSID__';
                return (
                  <WlanSelectorEntry
                    key={ssid}
                    entry={entry}
                    selected={selected === ssid}
                    onClick={() => setSelected(ssid)}
                  />
                );
              })}
            </div>
            <div className="wlan-selector-footer">
              <span onClick={() => invoke('open_file', { path: 'ms-settings:network' })}>
                {t('network.more')}
              </span>
            </div>
          </div>
        </>
      }
    >
      {children}
    </Popover>
  );
}
