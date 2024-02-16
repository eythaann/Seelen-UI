import { Header } from './components/header';
import { Navigation } from './components/navigation';
import { General } from './modules/general/infra';

import { useAppSelector } from './modules/shared/app/hooks';
import { selectRoute } from './modules/shared/app/selectors';

import { Route } from './modules/shared/domain/routes';

const ComponentByRout: Record<Route, React.JSXElementConstructor<any>> = {
  [Route.GENERAL]: General,
  [Route.MONITORS]: General,
  [Route.SHORTCUTS]: General,
  [Route.SPECIFIT_APPS]: General,
};

export function App() {
  let route = useAppSelector(selectRoute);
  let Component = ComponentByRout[route];

  return <>
    <Navigation />
    <Header />
    <div className="content">
      <Component />
    </div>
  </>;
}