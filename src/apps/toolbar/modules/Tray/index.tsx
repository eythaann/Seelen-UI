import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Popover } from 'antd';
import moment from 'moment';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { SeelenCommand, useWindowFocusChange } from 'seelen-core';
import { TrayTM } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra/infra';
import { LAZY_CONSTANTS } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';
import { FocusedApp } from 'src/apps/shared/interfaces/common';

import { TrayInfo } from '../shared/store/domain';

import { OverflowTooltip } from '../../../shared/components/OverflowTooltip';

interface Props {
  module: TrayTM;
}

function TrayItem(props: { tray: TrayInfo; onAction: anyFunction; idx: number }) {
  const { tray, onAction, idx } = props;

  const { t } = useTranslation();

  return (
    <li
      className="tray-item"
      onClick={((e) => {
        invoke(SeelenCommand.OnClickTrayIcon, { idx });
        onAction(false);

        e.preventDefault();
        e.stopPropagation();
      })}
      onContextMenu={(e) => {
        invoke(SeelenCommand.OnContextMenuTrayIcon, { idx });
        onAction(true);

        e.preventDefault();
        e.stopPropagation();
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
  const [currentFocus, setCurrentFocus] = useState(false);

  const trayList = useSelector(Selectors.systemTray);
  const focusedApp: FocusedApp | undefined = useSelector(Selectors.focused);
  let intervalId = useRef<any>(null);

  useEffect(() => {
    emit('register-tray-events');
  }, []);

  let blockUntil = useRef(moment(new Date()));
  useWindowFocusChange((focused) => {
    if (!focused && blockUntil.current < moment(new Date())) {
      setOpenPreview(false);
    }

    setCurrentFocus(focused);
  });

  useEffect(() => {
    if (!currentFocus && blockUntil.current < moment(new Date())) {
      setOpenPreview(false);
    }
  }, [focusedApp]);

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
        <BackgroundByLayersV2 className="tray" prefix="tray">
          <ul className="tray-list">
            {trayList.map((tray, idx) => (
              <TrayItem key={idx} idx={idx} tray={tray} onAction={(isContextMenu) => {
                if (isContextMenu) {
                  blockUntil.current = moment(new Date()).add(1, 's');
                }
              }} />
            ))}
          </ul>
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} />
    </Popover>
  );
}
