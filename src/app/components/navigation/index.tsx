import { memo } from 'react';

import { useAppSelector } from '../../modules/shared/app/hooks';
import { selectRoute } from '../../modules/shared/app/selectors';
import { cx } from '../../modules/shared/app/utils';

import { Route, RouteLabels } from '../../modules/shared/domain/routes';

import cs from './index.module.css';

export const Navigation = memo(() => {
  let current = useAppSelector(selectRoute);

  return <div className={cs.navigation}>
    {Object.values(Route).map((route) => {
      return <div className={cx({
        [cs.active!]: current === route,
      })} >{RouteLabels[route]}</div>;
    })}
  </div>;
});