import { Header } from './components/header';
import { Navigation } from './components/navigation';
import { Suspense } from 'react';

import { AppsConfiguration } from './modules/appsConfigurations/infra/infra';
import { General } from './modules/general/main/infra';
import { StylesView } from './modules/general/visuals/infra';
import { Information } from './modules/information/infrastructure';
import { Monitors } from './modules/monitors/main/infra';
import { SeelenWegSettings } from './modules/seelenweg/infra';
import { Shortcuts } from './modules/shortcuts/infrastructure';
import { Updates } from './modules/updates/infrastructure';

import { useAppSelector } from './modules/shared/app/hooks';
import { RootSelectors } from './modules/shared/app/selectors';

import { Route } from './modules/shared/domain/routes';

const ComponentByRout: Record<Route, React.JSXElementConstructor<any>> = {
  [Route.GENERAL]: General,
  [Route.MONITORS]: Monitors,
  [Route.STYLES]: StylesView,
  [Route.SHORTCUTS]: Shortcuts,
  [Route.SPECIFIT_APPS]: AppsConfiguration,
  [Route.INFO]: Information,
  [Route.UPDATES]: Updates,
  [Route.SEELEN_WEG]: SeelenWegSettings,
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
