import { memo, useCallback } from 'react';

import { useAppDispatch, useAppSelector } from '../../modules/shared/app/hooks';
import { mainActions } from '../../modules/shared/app/reducer';
import { selectRoute } from '../../modules/shared/app/selectors';
import { cx } from '../../modules/shared/app/utils';

import { Route, RouteLabels } from '../../modules/shared/domain/routes';

import cs from './index.module.css';

interface ItemProps {
  route: Route;
  isActive: boolean;
}

const Item = ({ route, isActive }: ItemProps) => {
  let dispatch = useAppDispatch();
  let onclick = useCallback(() => {
    dispatch(mainActions.setRoute(route));
  }, []);

  return <div
    onClick={onclick.bind(route)}
    className={cx({
      [cs.active!]: isActive,
    })}
  >
    {RouteLabels[route]}
  </div>;
};

export const Navigation = memo(() => {
  let current = useAppSelector(selectRoute);
  return <div className={cs.navigation}>
    {Object.values(Route).map((route) => {
      return <Item route={route} isActive={current === route} />;
    })}
  </div>;
});