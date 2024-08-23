import { OverflowTooltip } from '../../../shared/components/OverflowTooltip';
import { TrayTM } from '../../../shared/schemas/Placeholders';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Popover } from 'antd';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra';
import { useAppBlur } from '../shared/hooks/infra';
import { LAZY_CONSTANTS } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';

import { TrayInfo } from '../shared/store/domain';

interface Props {
  module: TrayTM;
}

function TrayItem(props: { tray: TrayInfo; onAction: anyFunction; idx: number }) {
  const { tray, onAction, idx } = props;

  const { t } = useTranslation();

  return (
    <li
      className="tray-item"
      onClick={() => {
        invoke('on_click_tray_icon', { idx });
        onAction();
      }}
      onContextMenu={() => {
        invoke('on_context_menu_tray_icon', { idx });
        onAction();
      }}
    >
      <div className="tray-item-icon">
        <img src={convertFileSrc(tray.icon ? tray.icon : LAZY_CONSTANTS.MISSING_ICON_PATH)} />
      </div>
      <OverflowTooltip
        overlayClassName="tray-item-label-tooltip"
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

  useAppBlur(() => {
    setOpenPreview(false);
  });

  useEffect(() => {
    if (openPreview) {
      intervalId.current = setInterval(() => {
        invoke('temp_get_by_event_tray_info');
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
        <BackgroundByLayersV2 className="tray" prefix="tray">
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
