import { SeelenCommand } from '@seelen-ui/lib';
import { WlanBssEntry } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { useWindowFocusChange } from '@shared/hooks';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { VNode } from 'preact';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../shared/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { AnimatedPopover } from '../../../../shared/components/AnimatedWrappers';
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
    <BackgroundByLayersV2
      className="wlan-selector"
      prefix="wlan-selector"
      onContextMenu={(e) => e.stopPropagation()}
    >
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
          <button className="wlan-selector-refresh">
            <Icon iconName="TbRefresh" size={12} />
          </button>
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
        <button
          className="wlan-selector-footer-button"
          onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:network' })}
        >
          {t('network.more')}
        </button>
      </div>
    </BackgroundByLayersV2>
  );
}

export interface WlanSelectorProperties {
  setActive: (value: boolean) => void;
  children: VNode;
}

export function WithWlanSelector({ setActive, children }: WlanSelectorProperties) {
  const [mounted, setMounted] = useState(false);
  const [openPreview, setOpenPreview] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (!mounted) {
      return;
    }
    setActive(openPreview);

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
    <AnimatedPopover
      animationDescription={{
        openAnimationName: 'wlan-open',
        closeAnimationName: 'wlan-close',
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      content={<WlanSelector open={openPreview} />}
    >
      {children}
    </AnimatedPopover>
  );
}
