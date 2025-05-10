import { Route, Routes } from 'react-router';

import { AppLauncherSettings } from './modules/AppLauncher/infra';
import { AppsConfiguration } from './modules/appsConfigurations/infra/infra';
import { SettingsByMonitor } from './modules/ByMonitor/infra';
import { DeveloperTools } from './modules/developer/infra';
import { FancyToolbarSettings } from './modules/fancyToolbar/infra';
import { General } from './modules/general/main/infra';
import { Information } from './modules/information/infrastructure';
import { ResourcesView } from './modules/resources/infra';
import { SeelenWegSettings } from './modules/seelenweg/infra';
import { Shortcuts } from './modules/shortcuts/infrastructure';
import { WallSettings } from './modules/Wall/infra';
import { WindowManagerSettings } from './modules/WindowManager/main/infra';

import { Layout } from './components/layout';
import { RoutePath } from './components/navigation/routes';
import { Home } from './modules/Home';

export function Routing() {
  return (
    <Routes>
      <Route Component={Layout}>
        <Route index Component={Home} />
        <Route path={RoutePath.General} Component={General} />
        <Route path={RoutePath.Resource + '/*'} Component={ResourcesView} />
        <Route path={RoutePath.SettingsByMonitor} Component={SettingsByMonitor} />
        <Route path={RoutePath.AppLauncher} Component={AppLauncherSettings} />
        <Route path={RoutePath.WallpaperManager} Component={WallSettings} />
        <Route path={RoutePath.Shortcuts} Component={Shortcuts} />
        <Route path={RoutePath.SettingsByApplication} Component={AppsConfiguration} />
        <Route path={RoutePath.Extras} Component={Information} />
        <Route path={RoutePath.SeelenWeg} Component={SeelenWegSettings} />
        <Route path={RoutePath.WindowManager} Component={WindowManagerSettings} />
        <Route path={RoutePath.FancyToolbar} Component={FancyToolbarSettings} />
        <Route path={RoutePath.DevTools} Component={DeveloperTools} />
        <Route path="widget/:username/:resourceName" Component={() => '🏗️🚧'} />
      </Route>
    </Routes>
  );
}
