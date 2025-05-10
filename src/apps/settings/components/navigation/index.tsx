import { Tooltip } from 'antd';
import { memo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { NavLink, useLocation } from 'react-router';

import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { cx } from '../../modules/shared/utils/app';
import { Icon } from 'src/apps/shared/components/Icon';

import { RouteIcons, RoutePath } from './routes';
import cs from './index.module.css';

const general = [
  RoutePath.Home,
  RoutePath.General,
  RoutePath.Resource,
  RoutePath.FancyToolbar,
  RoutePath.WindowManager,
  RoutePath.SeelenWeg,
  RoutePath.WallpaperManager,
  RoutePath.AppLauncher,
  RoutePath.Shortcuts,
];
const advanced = [RoutePath.SettingsByMonitor, RoutePath.SettingsByApplication];
const developer = [RoutePath.Mods, RoutePath.DevTools];

export const Navigation = memo(() => {
  const [collapsed, setCollapsed] = useState(false);

  let location = useLocation();
  let devTools = useAppSelector(RootSelectors.devTools);

  console.log(location);
  const Mapper = (route: RoutePath) => (
    <Item key={route} route={route} isActive={location.pathname === route} collapsed={collapsed} />
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
          key={RoutePath.Extras}
          route={RoutePath.Extras}
          isActive={location.pathname === RoutePath.Extras}
          collapsed={collapsed}
        />
      </div>
    </div>
  );
});

interface ItemProps {
  route: RoutePath;
  isActive: boolean;
  collapsed: boolean;
}

const Item = ({ route, isActive, collapsed }: ItemProps) => {
  const { t } = useTranslation();

  const key = route === '/' ? 'home' : route.replace('/', '');
  const label = t(`header.labels.${key}`);
  return (
    <Tooltip placement="right" title={collapsed ? label : null}>
      <NavLink
        to={route}
        className={cx(cs.item, {
          [cs.active!]: isActive,
        })}
      >
        {RouteIcons[route]}
        <span className={cs.label}>{label}</span>
      </NavLink>
    </Tooltip>
  );
};
