import { Route, Routes } from "react-router";

import { AppsConfiguration } from "./modules/appsConfigurations/infra/infra.tsx";
import { SettingsByMonitor } from "./modules/ByMonitor/infra/index.tsx";
import { DeveloperTools } from "./modules/developer/infra.tsx";
import { Information } from "./modules/extras/infrastructure.tsx";
import { FancyToolbarSettings } from "./modules/fancyToolbar/infra.tsx";
import { General } from "./modules/general/main/infra/index.tsx";
import { ResourcesView } from "./modules/resources/infra.tsx";
import { SeelenWegSettings } from "./modules/seelenweg/infra.tsx";
import { Shortcuts } from "./modules/shortcuts/infrastructure.tsx";
import { WallSettings } from "./modules/Wall/infra.tsx";
import { WindowManagerSettings } from "./modules/WindowManager/main/infra/index.tsx";

import { Layout } from "./components/layout/index.tsx";
import { RoutePath } from "./components/navigation/routes.tsx";
import { Home } from "./modules/Home/index.tsx";
import { IconPackEditorView } from "./modules/IconPackEditor/index.tsx";
import { ThemeView } from "./modules/resources/Theme/View.tsx";
import { SingleWallpaperView } from "./modules/resources/Wallpapers/View.tsx";
import { WidgetView } from "./modules/resources/Widget/View.tsx";

export function Routing() {
  return (
    <Routes>
      <Route Component={Layout}>
        <Route index Component={Home} />
        <Route path={RoutePath.General} Component={General} />
        <Route path={RoutePath.Resource + "/*"} Component={ResourcesView} />
        <Route path={RoutePath.SettingsByMonitor} Component={SettingsByMonitor} />
        <Route path={RoutePath.WallpaperManager} Component={WallSettings} />
        <Route path={RoutePath.Shortcuts} Component={Shortcuts} />
        <Route path={RoutePath.SettingsByApplication} Component={AppsConfiguration} />
        <Route path={RoutePath.Extras} Component={Information} />
        <Route path={RoutePath.SeelenWeg} Component={SeelenWegSettings} />
        <Route path={RoutePath.WindowManager} Component={WindowManagerSettings} />
        <Route path={RoutePath.FancyToolbar} Component={FancyToolbarSettings} />
        <Route path={RoutePath.DevTools} Component={DeveloperTools} />
        <Route path={RoutePath.IconPackEditor} Component={IconPackEditorView} />
        <Route path="widget" Component={WidgetView} />
        <Route path="theme" Component={ThemeView} />
        <Route path="wallpaper" Component={SingleWallpaperView} />
      </Route>
    </Routes>
  );
}
