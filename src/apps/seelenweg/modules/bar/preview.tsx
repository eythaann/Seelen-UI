import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin } from 'antd';
import { MouseEvent, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { Constants } from '../shared/utils/infra';
import cs from './infra.module.css';

import { SelectOpenApp, Selectors } from '../shared/store/app';

import { HWND } from '../shared/store/domain';

interface PreviewProps {
  hwnd: HWND;
}

export const WegPreview = ({ hwnd }: PreviewProps) => {
  const styles = useSelector(Selectors.theme.seelenweg.preview.items);
  const app = useSelector(SelectOpenApp(hwnd));
  const [imageSrc, setImageSrc] = useState<string | null>(null);

  useEffect(() => {
    const uslistener = listen(`weg-preview-update-${app?.process_hwnd || 0}`, () => {
      const postfix = `?${new Date().getTime()}`;
      setImageSrc(convertFileSrc(`${Constants.TEMP_FOLDER}${app?.process_hwnd || 0}.png`) + postfix);
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
        {imageSrc ? <img src={imageSrc} /> : <Spin />}
      </div>
    </div>
  );
};
