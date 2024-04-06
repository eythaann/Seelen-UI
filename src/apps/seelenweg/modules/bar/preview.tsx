import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin } from 'antd';
import { MouseEvent, useEffect, useReducer, useState } from 'react';
import { useSelector } from 'react-redux';

import { LAZY_CONSTANTS } from '../shared/utils/infra';
import cs from './infra.module.css';

import { SelectOpenApp, Selectors } from '../shared/store/app';

import { HWND } from '../shared/store/domain';

interface PreviewProps {
  hwnd: HWND;
}

export const WegPreview = ({ hwnd }: PreviewProps) => {
  const styles = useSelector(Selectors.theme.seelenweg.preview.items);
  const app = useSelector(SelectOpenApp(hwnd));

  const imageUrl = convertFileSrc(`${LAZY_CONSTANTS.TEMP_FOLDER}${app?.process_hwnd || 0}.png`);

  const [imageSrc, setImageSrc] = useState<string | null>(imageUrl);
  const [_, forceUpdate] = useReducer((x) => x + 1, 0);

  useEffect(() => {
    const uslistener = listen(`weg-preview-update-${app?.process_hwnd || 0}`, () => {
      setImageSrc(imageUrl);
      forceUpdate();
    });
    return () => {
      uslistener.then((unlisten) => unlisten());
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
      className={cs.preview}
      style={styles.content}
      onClick={() => invoke('weg_toggle_window_state', { hwnd: app.hwnd || 0, exePath: app.execution_path })}
    >
      <div className={cs.title} style={styles.title}>
        <div className={cs.label}>{app.title}</div>
        <div className={cs.close} onClick={onClose}>
          x
        </div>
      </div>
      <div className={cs.image} style={styles.image}>
        {imageSrc ? <img src={imageSrc + `?${new Date().getTime()}`} onError={() => setImageSrc(null)}/> : <Spin />}
      </div>
    </div>
  );
};
