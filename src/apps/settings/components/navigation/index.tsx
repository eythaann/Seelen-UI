import { ResourceText } from '@shared/components/ResourceText';
import { Tooltip } from 'antd';
import React, { memo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { NavLink, useLocation } from 'react-router';

import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { cx } from '../../modules/shared/utils/app';
import { Icon } from 'src/apps/shared/components/Icon';

import { RouteIcons, RoutePath } from './routes';
import cs from './index.module.css';

export const Navigation = memo(() => {
  const [collapsed, setCollapsed] = useState(false);

  const widgets = useSelector(RootSelectors.widgets);
  const devTools = useAppSelector(RootSelectors.devTools);

  const { t } = useTranslation();
  const location = useLocation();

  const Mapper = (route: RoutePath) => {
    return (
      <Item
        key={route}
        route={route}
        isActive={location.pathname.startsWith(route)}
        collapsed={collapsed}
        label={t(`header.labels.${route.replace('/', '')}`)}
        icon={RouteIcons[route]}
      />
    );
  };

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
        <div className={cs.group}>
          <Item
            route={RoutePath.Home}
            isActive={location.pathname === '/'}
            label={t('header.labels.home')}
            icon={<Icon iconName="TbHome" />}
            collapsed={collapsed}
          />
          {[RoutePath.General, RoutePath.Resource, RoutePath.Shortcuts].map(Mapper)}
        </div>

        <div className={cs.separator} />
        <div className={cs.group}>
          {widgets.map((widget) => (
            <Item
              key={widget.id}
              route={`/widget/${widget.id.replace('@', '')}`}
              isActive={location.pathname.startsWith(`/widget/${widget.id.replace('@', '')}`)}
              collapsed={collapsed}
              label={<ResourceText text={widget.metadata.displayName} />}
              icon={<Icon iconName={widget.icon as any || 'BiSolidWidget'} />}
            />
          ))}
        </div>

        <div className={cs.separator} />
        <div className={cs.group}>
          {[RoutePath.SettingsByMonitor, RoutePath.SettingsByApplication].map(Mapper)}
        </div>

        {devTools && (
          <>
            <div className={cs.separator} />
            <div className={cs.group}>{[RoutePath.DevTools].map(Mapper)}</div>
          </>
        )}
      </div>
      <div className={cs.footer}>{[RoutePath.Extras].map(Mapper)}</div>
    </div>
  );
});

interface ItemProps {
  route: string;
  isActive: boolean;
  collapsed: boolean;
  icon?: React.ReactNode;
  label: React.ReactNode;
}

const Item = ({ route, icon, label, isActive, collapsed }: ItemProps) => {
  return (
    <Tooltip placement="right" title={collapsed ? label : null}>
      <NavLink
        to={route}
        className={cx(cs.item, {
          [cs.active!]: isActive,
        })}
      >
        {icon}
        <span className={cs.label}>{label}</span>
      </NavLink>
    </Tooltip>
  );
};
