import { ConfigProvider, theme } from 'antd';
import { Suspense, useEffect } from 'react';
import { useSelector } from 'react-redux';

import { AppLauncherSettings } from './modules/AppLauncher/infra';
import { AppsConfiguration } from './modules/appsConfigurations/infra/infra';
import { SettingsByMonitor } from './modules/ByMonitor/infra';
import { DeveloperTools } from './modules/developer/infra';
import { FancyToolbarSettings } from './modules/fancyToolbar/infra';
import { General } from './modules/general/main/infra';
import { Information } from './modules/information/infrastructure';
import { SeelenWegSettings } from './modules/seelenweg/infra';
import { Shortcuts } from './modules/shortcuts/infrastructure';
import { WallSettings } from './modules/Wall/infra';
import { WindowManagerSettings } from './modules/WindowManager/main/infra';

import { newSelectors } from './modules/shared/store/app/reducer';
import { RootSelectors } from './modules/shared/store/app/selectors';

import { useDarkMode } from '../shared/styles';
import { Header } from './components/header';
import { Navigation } from './components/navigation';
import { Route } from './components/navigation/routes';
import { Home } from './modules/Home';

const ComponentByRout: Record<Route, React.JSXElementConstructor<any>> = {
  [Route.HOME]: Home,
  [Route.GENERAL]: General,
  [Route.MONITORS]: SettingsByMonitor,
  [Route.SEELEN_ROFI]: AppLauncherSettings,
  [Route.SEELEN_WALL]: WallSettings,
  [Route.SHORTCUTS]: Shortcuts,
  [Route.SPECIFIC_APPS]: AppsConfiguration,
  [Route.INFO]: Information,
  [Route.SEELEN_WEG]: SeelenWegSettings,
  [Route.SEELEN_WM]: WindowManagerSettings,
  [Route.SEELEN_BAR]: FancyToolbarSettings,
  [Route.DEVELOPER]: DeveloperTools,
};

export function App() {
  const isDarkMode = useDarkMode();
  const colors = useSelector(newSelectors.colors);
  let route = useSelector(RootSelectors.route);

  useEffect(() => {
    setTimeout(() => {
      let splashscreen = document.getElementById('splashscreen');
      splashscreen?.classList.add('vanish');
      setTimeout(() => splashscreen?.classList.add('hidden'), 300);
    }, 300);
  }, []);

  let Component = ComponentByRout[route];

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
      <Navigation />
      <Header />
      <div className="content">
        <Suspense fallback={<div>Loading...</div>}>
          <Component />
        </Suspense>
      </div>
    </ConfigProvider>
  );
}
