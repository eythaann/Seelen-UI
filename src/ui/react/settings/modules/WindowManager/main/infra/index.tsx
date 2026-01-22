import { SeelenWindowManagerWidgetId } from "@seelen-ui/lib";
import type { PluginId } from "@seelen-ui/lib/types";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { ConfigProvider, Select, Switch } from "antd";
import { useTranslation } from "react-i18next";

import { BorderSettings } from "../../border/infra.tsx";

import { getWmConfig, setWmDefaultLayout, setWmEnabled } from "../../application.ts";
import { plugins } from "../../../../state/resources.ts";

import { SettingsGroup, SettingsOption } from "../../../../components/SettingsBox/index.tsx";
import { WmAnimationsSettings } from "./Animations.tsx";
import { GlobalPaddings } from "./GlobalPaddings.tsx";
import { OthersConfigs } from "./Others.tsx";

export function WindowManagerSettings() {
  const wmSettings = getWmConfig();
  const defaultLayout = wmSettings.defaultLayout;

  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    setWmEnabled(value);
  };

  const onSelectLayout = (value: PluginId) => {
    setWmDefaultLayout(value);
  };

  const layouts = plugins.value.filter((plugin) => plugin.target === SeelenWindowManagerWidgetId);
  const usingLayout = layouts.find((plugin) => plugin.id === defaultLayout);

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t("wm.enable")}</b>
          </div>
          <Switch checked={wmSettings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <ConfigProvider componentDisabled={!wmSettings.enabled}>
        <SettingsGroup>
          <SettingsOption>
            <div>
              <b>{t("wm.layout")}:</b>
            </div>
            <Select
              style={{ width: "200px" }}
              value={defaultLayout}
              options={layouts.map((layout) => ({
                key: layout.id,
                label: <ResourceText text={layout.metadata.displayName} />,
                value: layout.id,
              }))}
              onSelect={onSelectLayout}
            />
          </SettingsOption>
          <div>
            <p>
              <b>{t("wm.description")}:</b>
              <ResourceText text={usingLayout?.metadata.description || "-"} />,
            </p>
          </div>
        </SettingsGroup>

        <GlobalPaddings />
        <BorderSettings />
        <WmAnimationsSettings />
        <OthersConfigs />
      </ConfigProvider>
    </>
  );
}
