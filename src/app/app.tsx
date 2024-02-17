import { Header } from './components/header';
import { Navigation } from './components/navigation';

import { AppsConfiguration } from './modules/appsConfigurations/infrastructure';
import { General } from './modules/general/main/infra';
import { Information } from './modules/information/infrastructure';
import { Monitors } from './modules/monitors/main/infra';
import { Shortcuts } from './modules/shortcuts/infrastructure';

import { useAppSelector } from './modules/shared/app/hooks';
import { RootSelectors } from './modules/shared/app/selectors';

import { Route } from './modules/shared/domain/routes';

const ComponentByRout: Record<Route, React.JSXElementConstructor<any>> = {
  [Route.GENERAL]: General,
  [Route.MONITORS]: Monitors,
  [Route.SHORTCUTS]: Shortcuts,
  [Route.SPECIFIT_APPS]: AppsConfiguration,
  [Route.INFO]: Information,
};

export function App() {
  let route = useAppSelector(RootSelectors.route);
  let Component = ComponentByRout[route];

  return <>
    <Navigation />
    <Header />
    <div className="content">
      <Component />
    </div>
  </>;
}