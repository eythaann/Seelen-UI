import { Tooltip } from 'antd';
import { memo, useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useAppDispatch, useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { cx } from '../../modules/shared/utils/app';
import { Icon } from 'src/apps/shared/components/Icon';

import { Route, RouteIcons } from './routes';
import cs from './index.module.css';

interface ItemProps {
  route: Route;
  isActive: boolean;
  collapsed: boolean;
}

const Item = ({ route, isActive, collapsed }: ItemProps) => {
  const { t } = useTranslation();

  let dispatch = useAppDispatch();
  let onclick = useCallback(() => {
    dispatch(RootActions.setRoute(route));
  }, []);

  let label = t(`header.labels.${route}`);
  return (
    <Tooltip placement="right" title={collapsed ? label : null}>
      <div
        onClick={onclick.bind(route)}
        className={cx(cs.item, {
          [cs.active!]: isActive,
        })}
      >
        {RouteIcons[route]}
        <span className={cs.label}>{label}</span>
      </div>
    </Tooltip>
  );
};

const general = [
  Route.HOME,
  Route.GENERAL,
  Route.SEELEN_BAR,
  Route.SEELEN_WM,
  Route.SEELEN_WEG,
  Route.SEELEN_WALL,
  Route.SEELEN_ROFI,
  Route.SHORTCUTS,
];
const advanced = [Route.MONITORS, Route.SPECIFIC_APPS];
const developer = [Route.MODS, Route.DEVELOPER];

export const Navigation = memo(() => {
  const [collapsed, setCollapsed] = useState(false);
  let current = useAppSelector(RootSelectors.route);
  let devTools = useAppSelector(RootSelectors.devTools);

  const Mapper = (route: Route) => (
    <Item key={route} route={route} isActive={current === route} collapsed={collapsed} />
  );

  return (
    <div
      className={cx(cs.navigation, {
        [cs.collapsed!]: collapsed,
      })}
    >
      <div className={cs.header}>
        <img src="./logo.svg" onClick={() => setCollapsed(!collapsed)} />
        <h1>Seelen UI</h1>
        <Icon
          className={cs.chevron}
          iconName="FaChevronLeft"
          onClick={() => setCollapsed(!collapsed)}
        />
      </div>
      <div className={cs.body}>
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
      <div className={cs.footer}>
        <Item
          key={Route.INFO}
          route={Route.INFO}
          isActive={current === Route.INFO}
          collapsed={collapsed}
        />
      </div>
    </div>
  );
});
