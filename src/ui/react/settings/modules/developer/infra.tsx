import { Switch, Tabs } from "antd";
import { useTranslation } from "react-i18next";

import {
  getDevTools,
  getUnlockShortcuts,
  getUnstableOptimizations,
  setDevTools,
  setUnlockShortcuts,
  setUnstableOptimizations,
} from "./application.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";
import { ColorPalette } from "./ColorPalette.tsx";
import { DevToolsSettings } from "./DevToolsSettings.tsx";
import { SystemIconPackView } from "./CachedIcons.tsx";
import { SharedComponents } from "./SharedComponents.tsx";
import { WidgetsDebug } from "./WidgetsDebug.tsx";

export function DeveloperTools() {
  const devTools = getDevTools();
  const unlockShortcuts = getUnlockShortcuts();
  const unstableOptimizations = getUnstableOptimizations();

  const { t } = useTranslation();

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={t("devtools.enable")}
          description={t("devtools.enable_description")}
          action={<Switch value={devTools} onChange={setDevTools} />}
        />
      </SettingsGroup>

      {devTools && (
        <SettingsGroup>
          <SettingsOption
            label={t("devtools.unstable_optimizations")}
            description={t("devtools.unstable_optimizations_description")}
            action={<Switch value={unstableOptimizations} onChange={setUnstableOptimizations} />}
          />
          <SettingsOption
            label={t("devtools.unlock_shortcuts")}
            description={t("devtools.unlock_shortcuts_description")}
            action={<Switch value={unlockShortcuts} onChange={setUnlockShortcuts} />}
          />
        </SettingsGroup>
      )}

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
