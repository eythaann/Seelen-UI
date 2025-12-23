import { SeelenCommand } from "@seelen-ui/lib";
import { type Plugin, ResourceKind } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { path } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "antd";
import React from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import cs from "./infra.module.css";

import { RootSelectors } from "../shared/store/app/selectors.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";
import { ResourceCard } from "./ResourceCard.tsx";

export function PluginsView() {
  const widgets = useSelector(RootSelectors.widgets);
  const plugins = useSelector(RootSelectors.plugins);

  const { t } = useTranslation();

  function targetLabel(target: string) {
    const widget = widgets.find((w) => w.id === target);
    if (widget) {
      return <ResourceText text={widget.metadata.displayName} />;
    }
    return <span>{target}</span>;
  }

  const groupedByTarget = plugins.reduce((acc, plugin) => {
    acc[plugin.target] ??= {
      label: targetLabel(plugin.target),
      plugins: [],
    };
    acc[plugin.target]!.plugins.push(plugin);
    return acc;
  }, {} as Record<string, { label: React.ReactNode; plugins: Plugin[] }>);

  Object.values(groupedByTarget).forEach((group) => {
    group.plugins.sort((a, b) => a.id.localeCompare(b.id));
  });

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("resources.open_folder")}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, {
                path: await path.join(dataDir, "plugins"),
              });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t("resources.discover")}:</span>
          <Button href="https://seelen.io/resources/s?category=Plugin" target="_blank" type="link">
            https://seelen.io/resources/s?category=Plugin
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        {Object.values(groupedByTarget).map((group, idx) => (
          <React.Fragment key={idx}>
            <b>{group.label}</b>
            {group.plugins.map((plugin) => (
              <ResourceCard
                key={plugin.id}
                resource={plugin}
                kind={ResourceKind.Plugin}
              />
            ))}
          </React.Fragment>
        ))}
      </div>
    </>
  );
}
