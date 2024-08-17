import { Icon } from '../../../../shared/components/Icon';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin } from 'antd';
import { MouseEvent, useEffect, useReducer, useState } from 'react';
import { useSelector } from 'react-redux';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { SelectOpenApp } from '../../shared/store/app';

import { HWND } from '../../shared/store/domain';

interface PreviewProps {
  hwnd: HWND;
}

export const UserApplicationPreview = ({ hwnd }: PreviewProps) => {
  const app = useSelector(SelectOpenApp(hwnd));

  const imageUrl = convertFileSrc(`${LAZY_CONSTANTS.TEMP_FOLDER}${app?.process_hwnd || 0}.png`);

  const [imageSrc, setImageSrc] = useState<string | null>(imageUrl);
  const [_, forceUpdate] = useReducer((x) => x + 1, 0);

  useEffect(() => {
    const unlisten = listen(`weg-preview-update-${app?.process_hwnd || 0}`, () => {
      setImageSrc(imageUrl);
      forceUpdate();
    });
    return () => {
      unlisten.then((unlisten) => unlisten()).catch(console.error);
    };
  }, []);

  const onClose = (e: MouseEvent) => {
    e.stopPropagation();
    invoke('weg_close_app', { hwnd });
  };

  if (!app) {
    return null;
  }

  return (
    <div
      className="weg-item-preview"
      onClick={() =>
        invoke('weg_toggle_window_state', { hwnd: app.hwnd || 0, exePath: app.execution_path })
      }
    >
      <div className="weg-item-preview-topbar">
        <div className="weg-item-preview-title">{app.title}</div>
        <div className="weg-item-preview-close" onClick={onClose}>
          <Icon iconName="IoClose" />
        </div>
      </div>
      <div className="weg-item-preview-image-container">
        {imageSrc ? (
          <img
            className="weg-item-preview-image"
            src={imageSrc + `?${new Date().getTime()}`}
            onError={() => setImageSrc(null)}
          />
        ) : (
          <Spin className="weg-item-preview-spin" />
        )}
      </div>
    </div>
  );
};
