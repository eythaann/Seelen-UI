import { TrayTM } from '../../../shared/schemas/Placeholders';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Popover } from 'antd';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra';
import { useAppBlur } from '../shared/hooks/infra';
import { LAZY_CONSTANTS } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: TrayTM;
}

export function TrayModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);

  const trayList = useSelector(Selectors.systemTray);

  useEffect(() => {
    emit('register-tray-events');
  }, []);

  useAppBlur(() => {
    setOpenPreview(false);
  });

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={(open) => {
        if (open) {
          invoke('temp_get_by_event_tray_info');
        }
        setOpenPreview(open);
      }}
      arrow={false}
      content={
        <BackgroundByLayersV2 className="tray" prefix="tray">
          <ul className="tray-list">
            {trayList.map((tray, idx) => (
              <li
                key={idx}
                className="tray-item"
                onClick={() => {
                  invoke('on_click_tray_icon', { idx });
                  setOpenPreview(false);
                }}
                onContextMenu={() => invoke('on_context_menu_tray_icon', { idx })}
              >
                <div className="tray-item-icon">
                  <img
                    src={convertFileSrc(tray.icon ? tray.icon : LAZY_CONSTANTS.MISSING_ICON_PATH)}
                  />
                </div>
                <div className="tray-item-label">{tray.label}</div>
              </li>
            ))}
          </ul>
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} />
    </Popover>
  );
}
