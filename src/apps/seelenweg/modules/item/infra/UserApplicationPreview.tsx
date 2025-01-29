import { SeelenCommand } from '@seelen-ui/lib';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin } from 'antd';
import { MouseEvent, useEffect, useReducer, useState } from 'react';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { HWND } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';

interface PreviewProps {
  title: string;
  hwnd: HWND;
  isFocused: boolean;
}

export const UserApplicationPreview = ({ title, hwnd, isFocused }: PreviewProps) => {
  const imageUrl = convertFileSrc(`${LAZY_CONSTANTS.TEMP_FOLDER}${hwnd}.png`);

  const [imageSrc, setImageSrc] = useState<string | null>(imageUrl);
  const [_, forceUpdate] = useReducer((x) => x + 1, 0);

  useEffect(() => {
    const unlisten = listen(`weg-preview-update-${hwnd}`, () => {
      setImageSrc(imageUrl);
      forceUpdate();
    });
    return () => {
      unlisten.then((unlisten) => unlisten()).catch(console.error);
    };
  }, []);

  const onClose = (e: MouseEvent) => {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  };

  return (
    <div
      className="weg-item-preview"
      onClick={() => {
        invoke(SeelenCommand.WegToggleWindowState, { hwnd, wasFocused: isFocused });
      }}
    >
      <div className="weg-item-preview-topbar">
        <div className="weg-item-preview-title">{title}</div>
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
