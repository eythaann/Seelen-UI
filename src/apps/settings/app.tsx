import { Header } from './components/header';
import { Navigation } from './components/navigation';
import { Route } from './components/navigation/routes';
import { Suspense } from 'react';

import { AppsConfiguration } from './modules/appsConfigurations/infra/infra';
import { General } from './modules/general/main/infra';
import { Information } from './modules/information/infrastructure';
import { Monitors } from './modules/monitors/main/infra';
import { SeelenWegSettings } from './modules/seelenweg/infra';
import { useAppSelector } from './modules/shared/utils/infra';
import { Shortcuts } from './modules/shortcuts/infrastructure';
import { WindowManagerSettings } from './modules/WindowManager/main/infra';

import { RootSelectors } from './modules/shared/store/app/selectors';

const ComponentByRout: Record<Route, React.JSXElementConstructor<any>> = {
  [Route.GENERAL]: General,
  [Route.MONITORS]: Monitors,
  [Route.SHORTCUTS]: Shortcuts,
  [Route.SPECIFIT_APPS]: AppsConfiguration,
  [Route.INFO]: Information,
  [Route.SEELEN_WEG]: SeelenWegSettings,
  [Route.SEELEN_WM]: WindowManagerSettings,
};

export function App() {
  let route = useAppSelector(RootSelectors.route);
  let Component = ComponentByRout[route];

  return (
    <>
      <Navigation />
      <Header />
      <div className="content">
        <Suspense fallback={<div>Loading...</div>}>
          <Component />
        </Suspense>
      </div>
    </>
  );
}
