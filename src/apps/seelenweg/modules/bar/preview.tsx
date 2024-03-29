import { invoke } from '@tauri-apps/api/core';
import { MouseEvent } from 'react';
import { useSelector } from 'react-redux';

import cs from './infra.module.css';

import { SelectOpenApp, Selectors } from '../shared/store/app';

import { HWND } from '../shared/store/domain';

interface PreviewProps {
  hwnd: HWND;
}

export const WegPreview = ({ hwnd }: PreviewProps) => {
  const styles = useSelector(Selectors.theme.seelenweg.preview.items);
  const app = useSelector(SelectOpenApp(hwnd));

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
        <div className={cs.close} onClick={onClose}>x</div>
      </div>
      <div className={cs.image} style={styles.image}>
        Preview
      </div>
    </div>
  );
};
