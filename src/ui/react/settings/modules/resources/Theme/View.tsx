import type { ThemeConfigDefinition as ThemeConfigDef, ThemeVariableDefinition } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Button, Empty, Input } from "antd";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router";
import { useMemo, useState } from "preact/hooks";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";

import { ThemeConfigDefinition } from "./components/ThemeConfigDefinition.tsx";
import { ResourceDescription } from "../ResourceCard.tsx";
import { resetThemeVariables } from "./application.ts";
import { themes, widgets } from "../../../state/resources.ts";
import cs from "../infra.module.css";

function getResourceSearchText(text: unknown): string {
  if (!text) return "";
  if (typeof text === "string") return text;
  if (typeof text === "object") {
    return Object.values(text as Record<string, string>).join(" ");
  }
  return "";
}

function matchesSetting(def: ThemeVariableDefinition, q: string): boolean {
  const label = getResourceSearchText(def.label).toLowerCase();
  const desc = getResourceSearchText(def.description).toLowerCase();
  const tip = getResourceSearchText(def.tip).toLowerCase();
  const name = (def.name || "").toLowerCase();
  return label.includes(q) || desc.includes(q) || tip.includes(q) || name.includes(q);
}

function filterDefinitions(defs: ThemeConfigDef[], q: string): ThemeConfigDef[] {
  if (!q) return defs;
  const result: ThemeConfigDef[] = [];
  for (const item of defs) {
    if ("group" in item) {
      const headerText = getResourceSearchText(item.group.header).toLowerCase();
      if (headerText.includes(q)) {
        result.push(item);
      } else {
        const filteredChildren = filterDefinitions(item.group.items, q);
        if (filteredChildren.length > 0) {
          result.push({
            group: {
              ...item.group,
              items: filteredChildren,
            },
          });
        }
      }
    } else {
      if (matchesSetting(item, q)) {
        result.push(item);
      }
    }
  }
  return result;
}

function countTotalSettings(defs: ThemeConfigDef[]): number {
  let count = 0;
  for (const item of defs) {
    if ("group" in item) {
      count += countTotalSettings(item.group.items);
    } else {
      count += 1;
    }
  }
  return count;
}

export function ThemeView() {
  const { t } = useTranslation();

  const [searchParams] = useSearchParams();
  const id = searchParams.get("id");

  const [searchQuery, setSearchQuery] = useState("");

  const theme = themes.value.find((t) => t.id === id);

  const trimmedQuery = searchQuery.trim().toLowerCase();
  const totalCount = useMemo(() => (theme ? countTotalSettings(theme.settings) : 0), [theme?.settings]);
  const filteredSettings = useMemo(
    () => (theme ? filterDefinitions(theme.settings, trimmedQuery) : []),
    [theme?.settings, trimmedQuery],
  );
  const filteredCount = useMemo(() => countTotalSettings(filteredSettings), [filteredSettings]);

  const handleReset = () => {
    if (theme) {
      resetThemeVariables(theme.id);
    }
  };

  if (!theme) {
    return <div>wow 404 !?</div>;
  }

  const affectedWidgets = Object.keys(theme.styles)
    .map((widgetId) => widgets.value.find((w) => w.id === widgetId)!)
    .filter(Boolean);

  return (
    <>
      <SettingsGroup>
        <ResourceDescription text={theme.metadata.description} />
      </SettingsGroup>

      {affectedWidgets.length > 0 && (
        <SettingsGroup>
          <SettingsSubGroup label={t("resources.affected_widgets")}>
            <div className={cs.tags}>
              {affectedWidgets.map((widget) => (
                <div key={widget.id} className={cs.tag}>
                  <ResourceText text={widget.metadata.displayName} />
                </div>
              ))}
            </div>
          </SettingsSubGroup>
        </SettingsGroup>
      )}

      <SettingsGroup>
        <SettingsOption
          label={t("reset_all_to_default")}
          action={
            <Button onClick={handleReset}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          }
        />
      </SettingsGroup>

      {totalCount > 3 && (
        <SettingsGroup>
          <div style={{ padding: "8px 12px" }}>
            <Input
              allowClear
              prefix={<Icon iconName="BiSearch" style={{ opacity: 0.5, marginRight: 4 }} />}
              placeholder={t("search")}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.currentTarget.value)}
            />
            {trimmedQuery && (
              <div style={{ marginTop: 6, fontSize: 12, opacity: 0.6 }}>
                {filteredCount} / {totalCount}
              </div>
            )}
          </div>
        </SettingsGroup>
      )}

      {filteredSettings.length > 0
        ? (
          filteredSettings.map((def, idx) => <ThemeConfigDefinition key={idx} themeId={theme.id} def={def} />)
        )
        : (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            style={{ margin: "24px 0" }}
          />
        )}
    </>
  );
}
