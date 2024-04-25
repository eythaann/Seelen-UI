import { Tooltip } from 'antd';
import { memo, useCallback } from 'react';

import { useAppDispatch, useAppSelector } from '../../modules/shared/utils/infra';
import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { cx } from '../../modules/shared/utils/app';

import {
  Route,
  RouteIcons,
  RouteLabels,
  WorkingInProgressRoutes,
} from './routes';

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
    };
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
      <Tooltip title={WorkingInProgressRoutes.includes(route) ? 'Working in progress' : undefined}>
        <span className={cs.label}>{RouteLabels[route]}</span>
      </Tooltip>
    </div>
  );
};

export const Navigation = memo(() => {
  let current = useAppSelector(RootSelectors.route);
  return (
    <div
      className={cx(cs.navigation, {
        [cs.tableView!]: current === Route.SPECIFIT_APPS,
      })}
    >
      <div className={cs.group}>
        {Object.values(Route).map((route) => {
          return route === Route.INFO ? null : (
            <Item key={route} route={route} isActive={current === route} />
          );
        })}
      </div>
      <Item key={Route.INFO} route={Route.INFO} isActive={current === Route.INFO} />
    </div>
  );
});
