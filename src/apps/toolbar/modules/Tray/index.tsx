import { SeelenCommand } from '@seelen-ui/lib';
import { TrayToolbarItem } from '@seelen-ui/lib/types';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Popover } from 'antd';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra/infra';
import { LAZY_CONSTANTS } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { TrayInfo } from '../shared/store/domain';

import { OverflowTooltip } from '../../../shared/components/OverflowTooltip';

interface Props {
  module: TrayToolbarItem;
}

function TrayItem(props: { tray: TrayInfo; onAction: anyFunction; idx: number }) {
  const { tray, onAction, idx } = props;

  const { t } = useTranslation();

  return (
    <li
      className="tray-item"
      onClick={() => {
        invoke(SeelenCommand.OnClickTrayIcon, { idx });
        onAction();
      }}
      onContextMenu={() => {
        invoke(SeelenCommand.OnContextMenuTrayIcon, { idx });
        onAction();
      }}
    >
      <div className="tray-item-icon">
        <img src={convertFileSrc(tray.icon ? tray.icon : LAZY_CONSTANTS.MISSING_ICON_PATH)} />
      </div>
      <OverflowTooltip
        rootClassName="tray-item-label-tooltip"
        className="tray-item-label"
        text={tray.label || t('unlabelled_tray')}
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
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <BackgroundByLayersV2 className="tray" prefix="tray" onContextMenu={(e) => e.stopPropagation()}>
          <ul className="tray-list">
            {trayList.map((tray, idx) => (
              <TrayItem key={idx} idx={idx} tray={tray} onAction={() => setOpenPreview(false)} />
            ))}
          </ul>
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} />
    </Popover>
  );
}
