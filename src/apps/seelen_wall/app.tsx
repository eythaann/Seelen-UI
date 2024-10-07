import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAnimate } from 'framer-motion';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';
import { useInterval } from 'seelen-core';

import { ThemedWallpaper, Wallpaper } from './modules/wallpaper/infra';

import { Selectors } from './modules/shared/store/app';

export function App() {
  const [currentBg, setCurrentBg] = useState(0);
  const [scope, animate] = useAnimate<HTMLDivElement>();

  const { backgrounds, interval } = useSelector(Selectors.settings);

  useInterval(() => {
    if (backgrounds.length > 1) {
      animate(scope.current, { opacity: 0.1 }).then(() => {
        setCurrentBg((currentIdx) => currentIdx + 1);
      });
    }
  }, interval * 1000);

  useEffect(() => {
    getCurrentWindow().show();
  }, []);

  const background = backgrounds[currentBg % backgrounds.length];
  if (!background) {
    return <ThemedWallpaper />;
  }
  return (
    <Wallpaper
      path={background.path}
      containerRef={scope}
      onLoad={() => {
        animate(scope.current, { opacity: 1 });
      }}
      onError={() => {
        setCurrentBg((currentIdx) => currentIdx + 1);
      }}
    />
  );
}
