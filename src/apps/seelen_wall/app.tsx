import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';
import { useInterval } from 'seelen-core';

import { Wallpaper } from './modules/wallpaper/infra';

import { Selectors } from './modules/shared/store/app';

export function App() {
  const [currentBg, setCurrentBg] = useState(0);

  const { backgrounds, interval } = useSelector(Selectors.settings);

  useInterval(() => setCurrentBg((currentIdx) => currentIdx + 1), interval * 1000);

  useEffect(() => {
    getCurrentWindow().show();
  }, []);

  const background = backgrounds[currentBg % backgrounds.length];
  return <Wallpaper path={background?.path || ''} />;
}
