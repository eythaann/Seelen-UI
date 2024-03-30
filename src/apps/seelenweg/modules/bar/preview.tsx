import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Spin } from 'antd';
import { MouseEvent, useEffect, useReducer, useState } from 'react';
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
  const [url, setUrl] = useState<string | null>(null);
  const [, forceUpdate] = useReducer((x) => x + 1, 0);

  useEffect(() => {
    const uslistener = listen(`weg-preview-update-${hwnd}`, () => {
      setUrl(convertFileSrc(`${Constants.TEMP_FOLDER}${hwnd}.png`));
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

  return (
    <div
      className={cs.preview}
      style={styles.content}
      onClick={() => invoke('weg_toggle_window_state', { hwnd, exePath: app?.exe || '' })}
    >
      <div className={cs.title} style={styles.title}>
        <div className={cs.label}>{app?.title}</div>
        <div className={cs.close} onClick={onClose}>
          x
        </div>
      </div>
      <div className={cs.image} style={styles.image}>
        {url ? <img src={`${url}?${new Date().getTime()}`} /> : <Spin />}
      </div>
    </div>
  );
};
