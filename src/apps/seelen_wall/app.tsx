import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAnimate } from 'framer-motion';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { ThemedWallpaper, Wallpaper } from './modules/wallpaper/infra';

import { Selectors } from './modules/shared/store/app';

import { useInterval } from '../shared/hooks';

export function App() {
  const [scope, animate] = useAnimate<HTMLDivElement>();

  const version = useSelector(Selectors.version);
  const { backgrounds, interval, randomize } = useSelector(Selectors.settings);

  const [currentBg, setCurrentBg] = useState(
    randomize ? Math.floor(Math.random() * backgrounds.length) : 0,
  );

  useInterval(
    () => {
      if (backgrounds.length > 1) {
        animate(scope.current, { opacity: 0.1 }).then(() => {
          if (randomize) {
            setCurrentBg(Math.floor(Math.random() * (backgrounds.length - 1)));
          } else {
            setCurrentBg((currentIdx) => (currentIdx + 1) % backgrounds.length);
          }
        });
      }
    },
    Number((interval < 1 ? 1 : interval) * 1000),
    [randomize, backgrounds.length],
  );

  useEffect(() => {
    getCurrentWindow().show();
  }, []);

  const background = backgrounds[currentBg];
  if (!background) {
    return <ThemedWallpaper />;
  }

  return (
    <Wallpaper
      key={version}
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
