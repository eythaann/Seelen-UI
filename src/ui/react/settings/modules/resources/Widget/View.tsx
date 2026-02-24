import type { WidgetId } from "@seelen-ui/lib/types";
import { Switch } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router";

import {
  getMonitorWidgetConfig,
  getWidgetConfig,
  patchWidgetConfig as patchWidget,
  patchWidgetInstanceConfig as patchInstance,
  patchWidgetMonitorConfig as patchMonitor,
} from "./application.ts";
import { widgets } from "../../../state/resources.ts";
import { getDevTools } from "../../developer/application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { ResourceDescription } from "../ResourceCard.tsx";
import { RenderBySettingsDeclaration } from "./ConfigRenderer.tsx";
import { WidgetInstanceSelector } from "./InstanceSelector.tsx";
import { SeelenWegSettings } from "../../seelenweg/infra.tsx";
import { WindowManagerSettings } from "../../WindowManager/main/infra/index.tsx";
import { FancyToolbarSettings } from "../../fancyToolbar/infra.tsx";
import { WallSettings } from "../../Wall/infra.tsx";

export function WidgetConfiguration({
  widgetId,
  monitorId,
}: {
  widgetId: WidgetId;
  monitorId?: string;
}) {
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null);

  const widget = widgets.value.find((t) => t.id === widgetId);
  const rootConfig = getWidgetConfig(widgetId) || {
    enabled: widget?.loader !== "Legacy" && !!widget?.metadata.bundled,
  };

  const monitorConfig = monitorId ? getMonitorWidgetConfig(monitorId, widgetId) : undefined;
  const areDevToolsEnabled = getDevTools();

  const { t } = useTranslation();

  if (!widget) {
    return <div>404</div>;
  }

  const onConfigChange = (key: string, value: any) => {
    if (monitorId) {
      patchMonitor(monitorId, widgetId, { [key]: value });
      return;
    }

    // intances `enabled` always inherit from widget root config
    if (selectedInstance && key !== "enabled") {
      patchInstance(widgetId, selectedInstance, { [key]: value });
      return;
    }

    patchWidget(widgetId, { [key]: value });
  };

  const instances = Object.keys(rootConfig.$instances || {}).map((instanceId) => ({
    label: `Instance ${instanceId.slice(0, 6)}`,
    value: instanceId,
  }));

  const instanceConfig = selectedInstance ? rootConfig.$instances?.[selectedInstance] : undefined;
  const config = {
    ...rootConfig,
    ...(instanceConfig || {}),
    ...(monitorConfig || {}),
  };

  const showToggleEnabled = !monitorId || widget.instances === "ReplicaByMonitor";

  return (
    <>
      <SettingsGroup>
        <ResourceDescription text={widget.metadata.description} />
      </SettingsGroup>

      {showToggleEnabled && (
        <SettingsGroup>
          <SettingsOption>
            <b>{monitorId ? t("widget.enable_for_monitor") : t("widget.enable")}</b>
            <Switch
              checked={config.enabled}
              onChange={(value) => {
                onConfigChange("enabled", value);
              }}
            />
          </SettingsOption>
        </SettingsGroup>
      )}

      {widget.instances === "Multiple" && (
        <SettingsGroup>
          <SettingsOption>
            <b>{t("widget.instances")}</b>
            <WidgetInstanceSelector
              widgetId={widgetId}
              options={instances}
              selected={selectedInstance}
              onSelect={setSelectedInstance}
            />
          </SettingsOption>
        </SettingsGroup>
      )}

      <RenderBySettingsDeclaration
        definitions={widget.settings}
        values={config}
        onConfigChange={onConfigChange}
        isByMonitor={!!monitorId}
      />

      {areDevToolsEnabled && (
        <SettingsGroup>
          <SettingsSubGroup label={<b>Raw Config</b>}>
            <pre>{JSON.stringify(rootConfig, null, 2)}</pre>
          </SettingsSubGroup>
          {!!monitorId && (
            <SettingsSubGroup label={<b>Raw Monitor Patch</b>}>
              <pre>{monitorConfig ? JSON.stringify(monitorConfig, null, 2) : "Inherited"}</pre>
            </SettingsSubGroup>
          )}
        </SettingsGroup>
      )}
    </>
  );
}

export function WidgetView() {
  const [searchParams] = useSearchParams();
  const widgetId = searchParams.get("id") as WidgetId;

  if (widgetId === "@seelen/weg") {
    return <SeelenWegSettings />;
  }

  if (widgetId === "@seelen/window-manager") {
    return <WindowManagerSettings />;
  }

  if (widgetId === "@seelen/fancy-toolbar") {
    return <FancyToolbarSettings />;
  }

  if (widgetId === "@seelen/wallpaper-manager") {
    return <WallSettings />;
  }

  return <WidgetConfiguration widgetId={widgetId} />;
}
