import { useDarkMode } from '@shared/styles';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { newSelectors } from './modules/shared/store/app/reducer';

import { Routing } from './router';

export function App() {
  const isDarkMode = useDarkMode();
  const colors = useSelector(newSelectors.colors);

  useEffect(() => {
    setTimeout(() => {
      let splashscreen = document.getElementById('splashscreen');
      splashscreen?.classList.add('vanish');
      setTimeout(() => splashscreen?.classList.add('hidden'), 300);
    }, 300);
  }, []);

  return (
    <ConfigProvider
      componentSize="small"
      theme={{
        token: {
          colorPrimary: isDarkMode ? colors.accent_light : colors.accent_dark,
        },
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <Routing />
    </ConfigProvider>
  );
}
