import { useEffect, useRef, useState } from 'react';
import { useSelector } from 'react-redux';

import { Wallpaper } from './modules/wallpaper/infra';

import { Selectors } from './modules/shared/store/app';

function useInterval(cb: () => void, ms: number) {
  const ref = useRef<NodeJS.Timeout | null>(null);
  const clearLastInterval = () => {
    if (ref.current) {
      clearInterval(ref.current);
    }
  };
  useEffect(() => {
    clearLastInterval();
    ref.current = setInterval(cb, ms);
    return clearLastInterval;
  }, [ms]);
}

export function App() {
  const [currentBg, setCurrentBg] = useState(0);

  const { backgrounds, interval } = useSelector(Selectors.settings);

  useInterval(() => setCurrentBg((currentIdx) => currentIdx + 1), interval * 1000);

  const background = backgrounds[currentBg % backgrounds.length];
  return <Wallpaper path={background?.path || ''} />;
}
