import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect } from 'react';

import { MonitorContainers } from './modules/Monitor/infra';

export function App() {
  /* const [scope, animate] = useAnimate<HTMLDivElement>();

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
 */
  useEffect(() => {
    getCurrentWindow().show();
  }, []);

  return <MonitorContainers />;
}
