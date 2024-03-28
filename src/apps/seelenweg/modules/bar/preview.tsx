import { invoke } from '@tauri-apps/api/core';
import { useSelector } from 'react-redux';

import cs from './infra.module.css';

import { Selectors } from '../shared/store/app';

import { PinnedAppSubItem } from '../shared/store/domain';

interface PreviewProps extends PinnedAppSubItem {
  exe: string;
}

export const WegPreview = ({ hwnd, title, exe }: PreviewProps) => {
  const styles = useSelector(Selectors.theme.seelenweg.preview.items);
  return (
    <div
      className={cs.preview}
      style={styles.content}
      onClick={() => invoke('weg_toggle_window_state', { hwnd, exePath: exe })}
    >
      <div className={cs.title} style={styles.title}>
        {title}
      </div>
      <div className={cs.image} style={styles.image}>
        Incomming
      </div>
    </div>
  );
};
