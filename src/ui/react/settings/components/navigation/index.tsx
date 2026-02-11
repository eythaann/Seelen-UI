import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { getResourceText } from "libs/ui/react/utils/index.ts";
import { Tooltip } from "antd";
import { memo, useState } from "react";
import { useTranslation } from "react-i18next";
import { NavLink, useLocation } from "react-router";

import { cx } from "../../modules/shared/utils/app.ts";

import { RouteIcons, RoutePath } from "./routes.tsx";
import cs from "./index.module.css";
import { settings, themes, widgets } from "../../state/mod.ts";

export const Navigation = memo(() => {
  const [collapsed, setCollapsed] = useState(false);

  const activeThemes = settings.value.activeThemes;
  const devTools = settings.value.devTools;

  const { t, i18n } = useTranslation();
  const location = useLocation();

  const Mapper = (route: RoutePath | null) => {
    if (!route) return null;
    return (
      <Item
        key={route}
        route={route}
        isActive={location.pathname.startsWith(route)}
        collapsed={collapsed}
        label={t(`header.labels.${route.replace("/", "")}`)}
        icon={RouteIcons[route]}
      />
    );
  };

  const themesDirectAccess = themes.value.filter(
    (theme) => theme.settings.length && activeThemes.includes(theme.id),
  );

  const advanceGroup = [
    RoutePath.SettingsByMonitor,
    RoutePath.SettingsByApplication,
    RoutePath.Shortcuts,
  ];
  const devGroup = [RoutePath.DevTools];

  if (devTools) {
    devGroup.push(RoutePath.IconPackEditor);
  }

  return (
    <div
      className={cx(cs.navigation, {
        [cs.collapsed!]: collapsed,
      })}
    >
      <div className={cs.header}>
        <img src="./logo.svg" onClick={() => setCollapsed(!collapsed)} loading="lazy" />
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
            isActive={location.pathname === "/"}
            label={t("header.labels.home")}
            icon={<Icon iconName="TbHome" />}
            collapsed={collapsed}
          />
          {[RoutePath.General, RoutePath.Resource].map(Mapper)}
        </div>

        <div className={cs.separator} />
        <div className={cs.group}>
          {widgets.value
            .filter((widget) => !widget.hidden)
            .toSorted((a, b) => {
              const aName = getResourceText(a.metadata.displayName, i18n.language);
              const bName = getResourceText(b.metadata.displayName, i18n.language);
              return aName.localeCompare(bName, i18n.language);
            })
            .map((widget) => (
              <Item
                key={widget.id}
                route={`/widget?${new URLSearchParams({ id: widget.id })}`}
                isActive={location.pathname === `/widget?${new URLSearchParams({ id: widget.id })}`}
                collapsed={collapsed}
                label={<ResourceText text={widget.metadata.displayName} />}
                icon={<Icon iconName={(widget.icon as any) || "BiSolidWidget"} />}
              />
            ))}
        </div>

        {!!themesDirectAccess.length && (
          <>
            <div className={cs.separator} />
            <div className={cs.group}>
              {themesDirectAccess
                .toSorted((a, b) => {
                  const aName = getResourceText(a.metadata.displayName, i18n.language);
                  const bName = getResourceText(b.metadata.displayName, i18n.language);
                  return aName.localeCompare(bName, i18n.language);
                })
                .map((theme) => (
                  <Item
                    key={theme.id}
                    route={`/theme?${new URLSearchParams({ id: theme.id })}`}
                    isActive={location.pathname === `/theme?${new URLSearchParams({ id: theme.id })}`}
                    collapsed={collapsed}
                    label={<ResourceText text={theme.metadata.displayName} />}
                    icon={<Icon iconName="BiSolidPalette" />}
                  />
                ))}
            </div>
          </>
        )}

        <div className={cs.separator} />
        <div className={cs.group}>{advanceGroup.map(Mapper)}</div>

        <div className={cs.separator} />
        <div className={cs.group}>{devGroup.map(Mapper)}</div>
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
