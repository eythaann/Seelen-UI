import { convertFileSrc } from '@tauri-apps/api/core';
import { PropsWithChildren } from 'react';
import { useSelector } from 'react-redux';

import { newSelectors } from '../../modules/shared/store/app/reducer';

import cs from './index.module.css';

interface Props extends PropsWithChildren {}

export function Monitor({ children }: Props) {
  const wallpaper = useSelector(newSelectors.wallpaper);

  return (
    <div className={cs.monitor}>
      <div className={cs.screen}>
        {wallpaper ? <img className={cs.wallpaper} src={convertFileSrc(wallpaper)} /> : null}
        {children}
      </div>
    </div>
  );
}
