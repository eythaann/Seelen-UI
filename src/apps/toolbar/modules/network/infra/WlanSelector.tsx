import { invoke } from '@tauri-apps/api/core';
import { Popover } from 'antd';
import { AnimatePresence, motion } from 'framer-motion';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { SeelenCommand, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { NetworkAdapter } from '../../shared/store/domain';
import { WlanBssEntry } from '../domain';

import { WlanSelectorEntry, WlanSelectorEntryProps } from './WlanSelectorEntry';

function WlanSelector({ open }: { open: boolean }) {
  const [selected, setSelected] = useState<string | null>(null);
  const [connectedEntry, setConnectedEntry] = useState<WlanBssEntry | undefined>(undefined);
  const [connectedNetworkAdapter, setConnectedNetworkAdapter] = useState<NetworkAdapter | undefined>(undefined);
  const [pinnedProps, setPinnedProps] = useState<WlanSelectorEntryProps | undefined>(undefined);

  const entries: [WlanBssEntry] = useSelector(Selectors.wlanBssEntries);
  const networkAdapters: [NetworkAdapter] = useSelector(Selectors.networkAdapters);
  const defaultIp = useSelector(Selectors.networkLocalIp);
  const online = useSelector(Selectors.online);

  const { t } = useTranslation();

  useEffect(() => {
    if (!open) {
      setSelected(null);
    }
  }, [open]);

  useEffect(() => {
    setConnectedNetworkAdapter(networkAdapters.find((i) => i.ipv4 === defaultIp));
  }, [networkAdapters, defaultIp]);

  useEffect(() => {
    setConnectedEntry(entries.find((entry) => entry.connected));
  }, [online, entries, selected, connectedNetworkAdapter]);

  const createLanPinnedProprs = function (): void {
    setPinnedProps({
      entry: {
        ssid: connectedNetworkAdapter!.name,
        bssid: connectedNetworkAdapter!.name,
        channel_frequency: 0,
        signal: 0,
        connected: true,
        connected_channel: false,
      },
      className: 'wlan-entry-connected',
      selected: true,
      onClick: () => {},
      icon: (connectedNetworkAdapter!.type !== 'IEEE80211') ? 'FaComputer' : undefined,
      loading: (connectedNetworkAdapter!.type === 'IEEE80211') ? true : false,
      buttonDisabled: (connectedNetworkAdapter!.type === 'IEEE80211') ? false : true,
    });
  };

  useEffect(() => {
    // Priority matters! Online -> Lan -> Wifi -> Wifi not loaded
    if (!online) {
      setPinnedProps({
        entry: {
          ssid: t('placeholder.ethernet_disconnected'),
          bssid: t('placeholder.ethernet_disconnected'),
          channel_frequency: 0,
          signal: 0,
          connected: true,
          connected_channel: false,
        },
        className: 'wlan-entry-connected',
        selected: true,
        onClick: () => {},
        icon: 'TbWorldCancel',
        loading: false,
        buttonDisabled: true,
      });
    } else if (connectedNetworkAdapter && !connectedEntry) {
      createLanPinnedProprs();
    } else if (connectedEntry) {
      setPinnedProps({
        entry: connectedEntry,
        className: 'wlan-entry-connected',
        selected: true,
        onClick: () => {},
        buttonDisabled: false,
      });
    } else if (connectedNetworkAdapter) {
      createLanPinnedProprs();
    }
  }, [connectedNetworkAdapter, connectedEntry, online]);

  let ssids = new Set<string>();
  let filtered = entries
    .toSorted((a, b) => b.signal - a.signal)
    .filter((entry) => {
      let ssid = entry.ssid || '__HIDDEN_SSID__';
      if (ssids.has(ssid)) {
        return false;
      }
      ssids.add(ssid);
      return !entry.connected;
    });

  return (
    <div className="wlan-selector">
      <BackgroundByLayersV2 prefix="wlan-selector" />
      { pinnedProps &&
      <motion.div
        key={pinnedProps.entry.ssid}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        initial={{ opacity: 0 }}
        transition={{ duration: 0.4 }}
      >
        <WlanSelectorEntry {...pinnedProps} />
      </motion.div>
      }
      <AnimatePresence>
        <div className="wlan-selector-entries">
          {filtered.length === 0 && (
            <div className="wlan-selector-empty">{t('network.not_found')}</div>
          )}
          {filtered.map((entry) => {
            let ssid = entry.ssid || '__HIDDEN_SSID__';
            return (
              <motion.div
                key={entry.ssid}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                initial={{ opacity: 0 }}
                transition={{ duration: 0.4 }}
              >
                <WlanSelectorEntry
                  key={ssid}
                  entry={entry}
                  selected={selected === ssid}
                  onClick={() => setSelected(ssid)}
                />
              </motion.div>
            );
          })}
        </div>
      </AnimatePresence>
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
