import { invoke } from '@tauri-apps/api/core';
import { Popover } from 'antd';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { SeelenCommand, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { WlanSelectorEntry } from './WlanSelectorEntry';

function WlanSelector({ open }: { open: boolean }) {
  const [selected, setSelected] = useState<string | null>(null);

  const entries = useSelector(Selectors.wlanBssEntries);
  const { t } = useTranslation();

  useEffect(() => {
    if (!open) {
      setSelected(null);
    }
  }, [open]);

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
    <div className="wlan-selector">
      <BackgroundByLayersV2 prefix="wlan-selector" />
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
        <span onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:network' })}>
          {t('network.more')}
        </span>
      </div>
    </div>
  );
}

export function WithWlanSelector({ children }: PropsWithChildren) {
  const [mounted, setMounted] = useState(false);
  const [openPreview, setOpenPreview] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (!mounted) {
      return;
    }
    if (openPreview) {
      invoke(SeelenCommand.WlanStartScanning);
    } else {
      invoke(SeelenCommand.WlanStopScanning);
    }
  }, [openPreview]);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<WlanSelector open={openPreview} />}
    >
      {children}
    </Popover>
  );
}
