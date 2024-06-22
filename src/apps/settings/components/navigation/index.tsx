import { Route, RouteIcons, RouteLabels, WorkingInProgressRoutes } from './routes';
import { Tooltip } from 'antd';
import { memo, useCallback } from 'react';

import { useAppDispatch, useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { cx } from '../../modules/shared/utils/app';

import cs from './index.module.css';

interface ItemProps {
  route: Route;
  isActive: boolean;
}

const Item = ({ route, isActive }: ItemProps) => {
  let dispatch = useAppDispatch();
  let onclick = useCallback(() => {
    if (WorkingInProgressRoutes.includes(route)) {
      return;
    }
    dispatch(RootActions.setRoute(route));
  }, []);

  return (
    <div
      onClick={onclick.bind(route)}
      className={cx(cs.item, {
        [cs.active!]: isActive,
      })}
    >
      <span className={cs.icon}>{RouteIcons[route]}</span>
      <Tooltip title={WorkingInProgressRoutes.includes(route) ? 'Work in progress' : undefined}>
        <span className={cs.label}>{RouteLabels[route]}</span>
      </Tooltip>
    </div>
  );
};

export const Navigation = memo(() => {
  let current = useAppSelector(RootSelectors.route);
  let devTools = useAppSelector(RootSelectors.devTools);

  let routes = Object.values(Route).filter(
    (r) => (r !== Route.DEVELOPER || devTools) && r !== Route.INFO,
  );

  return (
    <div
      className={cx(cs.navigation, {
        [cs.tableView!]: current === Route.SPECIFIC_APPS,
      })}
    >
      <div className={cs.group}>
        {routes.map((route) => (
          <Item key={route} route={route} isActive={current === route} />
        ))}
      </div>
      <Item key={Route.INFO} route={Route.INFO} isActive={current === Route.INFO} />
    </div>
  );
});
