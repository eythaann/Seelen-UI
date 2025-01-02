import { ConnectedMonitor } from '@seelen-ui/lib';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAnimate } from 'framer-motion';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { ThemedWallpaper, Wallpaper } from './modules/wallpaper/infra';

import { Selectors } from './modules/shared/store/app';

import { useInterval } from '../shared/hooks';

export function App() {
  const [definitiveLeft, setDefinitiveLeft] = useState(0);
  const [definitiveTop, setDefinitiveTop] = useState(0);
  const [definitiveDpi, setDefinitiveDpi] = useState(0);

  const [scope, animate] = useAnimate<HTMLDivElement>();

  const version = useSelector(Selectors.version);
  const { backgrounds, interval, randomize } = useSelector(Selectors.settings);
  const monitors: ConnectedMonitor[] = useSelector(Selectors.monitors);

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

  useEffect(() => {
    setDefinitiveLeft(Math.min(...monitors.map((item: ConnectedMonitor) => item.fromLeft)) * -1);
    setDefinitiveTop(Math.min(...monitors.map((item: ConnectedMonitor) => item.fromTop)) * -1);
    setDefinitiveDpi(Math.max(...monitors.map((item: ConnectedMonitor) => item.dpi)));
  }, [monitors]);

  const background = backgrounds[currentBg];
  if (!background) {
    return <ThemedWallpaper />;
  }

  return (
    <div style={{ position: 'relative', width: '100%', height: '100%' }}>
      { background && definitiveDpi != 0 && monitors && monitors.map((item) =>
        <Wallpaper
          key={version + item.id}
          style={{ position: 'absolute', width: item.width / definitiveDpi, height: item.height / definitiveDpi, left: (definitiveLeft + item.fromLeft) / definitiveDpi, top: (definitiveTop + item.fromTop) / definitiveDpi }}
          path={background.path}
          containerRef={scope}
          onLoad={() => {
            animate(scope.current, { opacity: 1 });
          }}
          onError={() => {
            setCurrentBg((currentIdx) => currentIdx + 1);
          }}
        />)
      }
    </div>
  );
}
