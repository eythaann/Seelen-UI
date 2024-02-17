import { memo, useCallback } from 'react';

import { useAppDispatch, useAppSelector } from '../../modules/shared/app/hooks';
import { RootActions } from '../../modules/shared/app/reducer';
import { RootSelectors } from '../../modules/shared/app/selectors';
import { cx } from '../../modules/shared/app/utils';

import { Route, RouteIcons, RouteLabels } from '../../modules/shared/domain/routes';

import cs from './index.module.css';

interface ItemProps {
  route: Route;
  isActive: boolean;
}

const Item = ({ route, isActive }: ItemProps) => {
  let dispatch = useAppDispatch();
  let onclick = useCallback(() => {
    dispatch(RootActions.setRoute(route));
  }, []);

  return <div
    onClick={onclick.bind(route)}
    className={cx(cs.item, {
      [cs.active!]: isActive,
    })}
  >
    {RouteIcons[route]} {RouteLabels[route]}
  </div>;
};

export const Navigation = memo(() => {
  let current = useAppSelector(RootSelectors.route);
  return <div className={cs.navigation}>
    <div className={cs.group}>
      {Object.values(Route).map((route) => {
        return route === Route.INFO
          ? null
          : <Item key={route} route={route} isActive={current === route} />;
      })}
    </div>
    <Item key={Route.INFO} route={Route.INFO} isActive={current === Route.INFO} />
  </div>;
});