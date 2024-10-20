import { ConfigProvider, theme } from 'antd';
import { useSelector } from 'react-redux';

import { Launcher } from './modules/launcher/infra';

import { Selectors } from './modules/shared/store/app';

import { useDarkMode } from '../shared/styles';

export function App() {
  const isDarkMode = useDarkMode();
  const colors = useSelector(Selectors.colors);

  return <ConfigProvider
    theme={{
      token: {
        colorPrimary: isDarkMode ? colors.accent_light : colors.accent_dark,
      },
      algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
    }}
  >
    <Launcher/>
  </ConfigProvider>;
}