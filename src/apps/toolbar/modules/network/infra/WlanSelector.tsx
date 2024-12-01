import { invoke } from '@tauri-apps/api/core';
import { Button, Popover, Tooltip } from 'antd';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { SeelenCommand, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';
import { Icon } from 'src/apps/shared/components/Icon';

import { WlanBssEntry } from '../domain';

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

  let hidden = entries.filter((e) => !e.ssid).toSorted((a, b) => b.signal - a.signal);
  let grouped = entries.reduce((groups, entry) => {
    if (!entry.ssid) {
      return groups;
    }
    if (!groups[entry.ssid]) {
      groups[entry.ssid] = [entry];
      return groups;
    }
    groups[entry.ssid]!.push(entry);
    groups[entry.ssid]!.sort((e1, e2) => e2.signal - e1.signal);
    return groups;
  }, {} as Record<string, [WlanBssEntry, ...WlanBssEntry[]]>);

  let sorted = Object.values(grouped).toSorted((a, b) => {
    let signalA = Math.max(...a.map((e) => e.signal));
    let signalB = Math.max(...b.map((e) => e.signal));
    return signalB - signalA;
  });

  let connected = sorted.find((group) => group.some((e) => e.connected));
  let known = sorted.filter(
    (group) => group.every((e) => !e.connected) && group.some((e) => e.known),
  );
  let unknown = sorted.filter(
    (group) => group.every((e) => !e.connected) && group.every((e) => !e.known),
  );

  return (
    <div className="wlan-selector" onContextMenu={(e) => e.stopPropagation()}>
      <BackgroundByLayersV2 prefix="wlan-selector" />

      {connected && (
        <>
          <div className="wlan-selector-title">{t('network.connected')}</div>
          <div className="wlan-selector-entries">
            <WlanSelectorEntry
              group={connected}
              selected={!selected || selected === connected[0].ssid}
              onClick={() => setSelected(connected![0].ssid)}
            />
          </div>
        </>
      )}

      {known.length > 0 && (
        <>
          <div className="wlan-selector-title">{t('network.saved')}</div>
          <div className="wlan-selector-entries">
            {known.map((group) => {
              let ssid = group[0].ssid!;
              return (
                <WlanSelectorEntry
                  key={ssid}
                  group={group}
                  selected={selected === ssid}
                  onClick={() => setSelected(ssid)}
                />
              );
            })}
          </div>
        </>
      )}

      <div className="wlan-selector-title">
        <span>{t('network.available')}</span>
        <Tooltip title={t('network.scanning')}>
          <Button type="text" size="small">
            <Icon iconName="TfiReload" className="wlan-selector-refresh" size={12} />
          </Button>
        </Tooltip>
      </div>
      <div className="wlan-selector-entries">
        {unknown.length === 0 && (
          <div className="wlan-selector-empty">{t('network.not_found')}</div>
        )}
        {unknown.map((group) => {
          let ssid = group[0].ssid!;
          return (
            <WlanSelectorEntry
              key={ssid}
              group={group}
              selected={selected === ssid}
              onClick={() => setSelected(ssid)}
            />
          );
        })}
        {hidden.length > 0 && (
          <WlanSelectorEntry
            key="__HIDDEN_SSID__"
            group={hidden as [WlanBssEntry, ...WlanBssEntry[]]}
            selected={selected === '__HIDDEN_SSID__'}
            onClick={() => setSelected('__HIDDEN_SSID__')}
          />
        )}
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
      destroyTooltipOnHide
    >
      {children}
    </Popover>
  );
}
