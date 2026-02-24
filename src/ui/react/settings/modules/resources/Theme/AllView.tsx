import { ResourceKind, type Theme, type ThemeId, type Widget } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Switch, Tooltip } from "antd";
import { Reorder } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useState } from "preact/hooks";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { setActiveThemes, settings } from "../../../state/mod.ts";
import { themes, widgets } from "../../../state/resources.ts";

import { resolveDisplayName, ResourceCard, ResourceListHeader } from "../ResourceCard.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { cx } from "../../shared/utils/app.ts";

export function ThemesView() {
  const activeIds = settings.value.activeThemes;
  const { t, i18n } = useTranslation();
  const [search, setSearch] = useState("");

  function toggleTheme(themeId: ThemeId) {
    if (activeIds.includes(themeId)) {
      setActiveThemes(activeIds.filter((x) => x !== themeId));
    } else {
      setActiveThemes([...activeIds, themeId]);
    }
  }

  function onReorder(themes: ThemeId[]) {
    setActiveThemes(themes);
  }

  const allFiltered = search
    ? themes.value.filter((theme) =>
      resolveDisplayName(theme.metadata.displayName, i18n.language).toLowerCase().includes(search.toLowerCase())
    )
    : themes.value;

  const disabled: Theme[] = [];
  const enabled: Theme[] = [];
  for (const theme of allFiltered) {
    if (activeIds.includes(theme.id)) {
      enabled.push(theme);
    } else {
      disabled.push(theme);
    }
  }
  enabled.sort((a, b) => activeIds.indexOf(a.id) - activeIds.indexOf(b.id));

  return (
    <>
      <ResourceListHeader
        discoverUrl="https://seelen.io/resources/s?category=Theme"
        search={search}
        onSearch={setSearch}
      />

      <div className={cs.list}>
        <b>{t("general.theme.selected")}</b>
        <Reorder.Group values={activeIds} onReorder={onReorder} className={cs.reorderGroup}>
          {enabled.map((theme) => (
            <Reorder.Item key={theme.id} value={theme.id}>
              <ThemeItem
                key={theme.id}
                theme={theme}
                onToggle={() => toggleTheme(theme.id)}
                checked
                widgets={widgets.value}
              />
            </Reorder.Item>
          ))}
        </Reorder.Group>

        <b>{t("general.theme.available")}</b>
        {disabled.map((theme) => (
          <ThemeItem
            key={theme.id}
            theme={theme}
            onToggle={() => toggleTheme(theme.id)}
            checked={false}
            widgets={widgets.value}
          />
        ))}
      </div>
    </>
  );
}

interface ThemeItemProps {
  theme: Theme;
  onToggle: () => void;
  checked: boolean;
  widgets: Widget[];
}

function ThemeItem({ theme, checked, onToggle, widgets }: ThemeItemProps) {
  let query = new URLSearchParams();
  query.set("id", theme.id);

  const { t } = useTranslation();

  let gpuImpact = false;
  let affectedWidgets: Widget[] = [];

  for (const [widgetId, style] of Object.entries(theme.styles)) {
    const widget = widgets.find((x) => x.id === widgetId);
    if (widget) {
      affectedWidgets.push(widget);
    }

    if (
      style &&
      style.includes("@keyframes") &&
      style.includes("animation:") &&
      style.includes("infinite")
    ) {
      gpuImpact = true;
    }
  }

  return (
    <ResourceCard
      resource={theme}
      kind={ResourceKind.Theme}
      body={
        <div className={cs.tags}>
          {theme.id !== "@default/theme" && gpuImpact && (
            <Tooltip title={t("resources.high_impact")}>
              <div className={cx(cs.tag, cs.warn)}>
                <Icon iconName="HiCpuChip" />
              </div>
            </Tooltip>
          )}

          {affectedWidgets.map((widget) => (
            <div key={widget.id} className={cs.tag}>
              <ResourceText text={widget.metadata.displayName} />
            </div>
          ))}
        </div>
      }
      actions={
        <>
          {theme.settings.length > 0 && (
            <NavLink to={`/theme?${query}`}>
              <Button type="text">
                <Icon iconName="RiSettings4Fill" />
              </Button>
            </NavLink>
          )}
          {theme.id !== "@default/theme" && <Switch value={checked} onChange={onToggle} />}
        </>
      }
    />
  );
}
