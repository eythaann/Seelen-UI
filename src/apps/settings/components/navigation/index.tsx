import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

import { useAppDispatch, useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { cx } from '../../modules/shared/utils/app';

import { Route, RouteIcons } from './routes';
import cs from './index.module.css';

interface ItemProps {
  route: Route;
  isActive: boolean;
}

const Item = ({ route, isActive }: ItemProps) => {
  const { t } = useTranslation();

  let dispatch = useAppDispatch();
  let onclick = useCallback(() => {
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
      <span className={cs.label}>{t(`header.labels.${route}`)}</span>
    </div>
  );
};

export const Navigation = memo(() => {
  let current = useAppSelector(RootSelectors.route);
  let devTools = useAppSelector(RootSelectors.devTools);

  let general = [
    Route.GENERAL,
    Route.SEELEN_BAR,
    Route.SEELEN_WM,
    Route.SEELEN_WEG,
    Route.SEELEN_WALL,
    Route.SHORTCUTS,
  ];

  let advanced = [Route.MONITORS, Route.SPECIFIC_APPS];

  let developer = [Route.DEVELOPER];

  const Mapper = (route: Route) => <Item key={route} route={route} isActive={current === route} />;

  return (
    <div
      className={cx(cs.navigation, {
        [cs.tableView!]: current === Route.SPECIFIC_APPS,
      })}
    >
      <div className={cs.navigationMain}>
        <div className={cs.group}>{general.map(Mapper)}</div>
        <div className={cs.separator} />
        <div className={cs.group}>{advanced.map(Mapper)}</div>
        {devTools && (
          <>
            <div className={cs.separator} />
            <div className={cs.group}>{developer.map(Mapper)}</div>
          </>
        )}
      </div>
      <Item key={Route.INFO} route={Route.INFO} isActive={current === Route.INFO} />
    </div>
  );
});
