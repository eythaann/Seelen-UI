import { SeelenCommand } from '@seelen-ui/lib';
import { TrayToolbarItem } from '@seelen-ui/lib/types';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra/infra';
import { LAZY_CONSTANTS } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';
import { useIcon, useWindowFocusChange } from 'src/apps/shared/hooks';

import { TrayInfo } from '../shared/store/domain';

import { AnimatedPopover } from '../../../shared/components/AnimatedWrappers';
import { OverflowTooltip } from '../../../shared/components/OverflowTooltip';

interface Props {
  module: TrayToolbarItem;
}

function TrayItem(props: { tray: TrayInfo }) {
  const [disabled, setDisabled] = useState(false);
  const { tray } = props;

  let iconSrc =
    useIcon({ path: tray.registry.executablePath }) ||
    convertFileSrc(LAZY_CONSTANTS.MISSING_ICON_PATH);
  const { t } = useTranslation();

  if (tray.registry.executablePath.endsWith('explorer.exe') && tray.registry.iconSnapshot) {
    const base64String = btoa(String.fromCharCode(...tray.registry.iconSnapshot));
    iconSrc = `data:image/png;base64,${base64String}`;
  }

  return (
    <li
      className="tray-item"
      onClick={() => {
        if (!disabled) {
          invoke(SeelenCommand.OnClickTrayIcon, { key: tray.registry.key }).finally(() => {
            setDisabled(false);
          });
        }
        setDisabled(true);
      }}
      onContextMenu={() => {
        if (!disabled) {
          invoke(SeelenCommand.OnContextMenuTrayIcon, { key: tray.registry.key }).finally(() => {
            setDisabled(false);
          });
        }
        setDisabled(true);
      }}
    >
      <div className="tray-item-icon-container">
        <img className="tray-item-icon" src={iconSrc} />
      </div>
      <OverflowTooltip
        rootClassName="tray-item-label-tooltip"
        className="tray-item-label"
        text={
          tray.label ||
          tray.registry.initialTooltip ||
          tray.registry.executablePath.split('\\').pop() ||
          t('unlabelled_tray')
        }
        placement="left"
        arrow={false}
      />
    </li>
  );
}

export function TrayModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);

  const trayList = useSelector(Selectors.systemTray);
  let intervalId = useRef<any>(null);

  useEffect(() => {
    emit('register-tray-events');
  }, []);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  useEffect(() => {
    if (openPreview) {
      intervalId.current = setInterval(() => {
        invoke(SeelenCommand.TempGetByEventTrayInfo);
      }, 1000);
    } else if (intervalId) {
      clearInterval(intervalId.current);
      intervalId.current = null;
    }
    return () => clearInterval(intervalId.current);
  }, [openPreview]);

  return (
    <AnimatedPopover
      animationDescription={{
        maxAnimationTimeMs: 500,
        openAnimationName: 'tray-open',
        closeAnimationName: 'tray-close',
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <BackgroundByLayersV2
          className="tray"
          prefix="tray"
          onContextMenu={(e) => e.stopPropagation()}
        >
          <ul className="tray-list">
            {trayList.map((tray) => (
              <TrayItem key={tray.registry.key} tray={tray} />
            ))}
          </ul>
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} active={openPreview} />
    </AnimatedPopover>
  );
}
