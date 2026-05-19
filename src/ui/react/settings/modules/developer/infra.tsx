import { Switch, Tabs } from "antd";
import { useTranslation } from "react-i18next";

import { getDevTools, getUnstableOptimizations, setDevTools, setUnstableOptimizations } from "./application.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";
import { ColorPalette } from "./ColorPalette.tsx";
import { DevToolsSettings } from "./DevToolsSettings.tsx";
import { SystemIconPackView } from "./CachedIcons.tsx";
import { SharedComponents } from "./SharedComponents.tsx";
import { WidgetsDebug } from "./WidgetsDebug.tsx";
import { getWegConfig, patchWegConfig } from "../seelenweg/application.ts";

export function DeveloperTools() {
  const devTools = getDevTools();
  const showEndTask = getWegConfig().showEndTask;
  const unstableOptimizations = getUnstableOptimizations();

  const { t } = useTranslation();

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("devtools.enable")}</b>
          <Switch value={devTools} onChange={setDevTools} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("devtools.unstable_optimizations")}
          description={t("devtools.unstable_optimizations_description")}
          action={<Switch value={unstableOptimizations} onChange={setUnstableOptimizations} />}
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("weg.show_end_task")}</b>
          <Switch
            checked={showEndTask}
            onChange={(value) => patchWegConfig({ showEndTask: value })}
          />
        </SettingsOption>
      </SettingsGroup>

      {devTools && (
        <Tabs
          items={[
            {
              key: "settings",
              label: t("devtools.tools"),
              children: <DevToolsSettings />,
            },
            {
              key: "colors",
              label: t("devtools.color_palette"),
              children: <ColorPalette />,
            },
            {
              key: "shared_components",
              label: t("devtools.shared_components"),
              children: <SharedComponents />,
            },
            {
              key: "icon_pack",
              label: t("devtools.icons"),
              children: <SystemIconPackView />,
            },
            {
              key: "widgets_debug",
              label: t("devtools.widgets_debug.tab"),
              children: <WidgetsDebug />,
            },
          ]}
        />
      )}
    </>
  );
}
