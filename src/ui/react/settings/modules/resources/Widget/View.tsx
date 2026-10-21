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
  resetWidgetConfig,
  resetWidgetMonitorConfig,
} from "./application.ts";
import { widgets } from "../../../state/resources.ts";
import { getDevTools } from "../../developer/application.ts";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import cs from "../infra.module.css";

import { Button } from "antd";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { ResourceDescription } from "../ResourceCard.tsx";
import { RenderBySettingsDeclaration } from "./ConfigRenderer.tsx";
import { WidgetInstanceSelector } from "./InstanceSelector.tsx";
import { SeelenWegSettings } from "./seelenweg/infra.tsx";
import { FancyToolbarSettings } from "./fancyToolbar/infra.tsx";
import { WallSettings } from "./Wall/infra.tsx";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Note } from "../../../components/Note/index.tsx";
import { WindowManagerSettings } from "./WindowManager/main/infra/index.tsx";

interface Props {
  widgetId: WidgetId;
  monitorId?: string;
  children?: React.ReactNode;
}

export function WidgetConfiguration({ widgetId, monitorId, children }: Props) {
  if (monitorId) {
    return <WidgetConfigOnMonitor widgetId={widgetId} monitorId={monitorId} />;
  }
  return <RootWidgetConfiguration widgetId={widgetId}>{children}</RootWidgetConfiguration>;
}

function RootWidgetConfiguration({ widgetId, children }: Omit<Props, "monitorId">) {
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null);

  const widget = widgets.value.find((t) => t.id === widgetId);

  const rootConfig = getWidgetConfig(widgetId) || {
    enabled: widget?.loader !== "Legacy" && !!widget?.metadata.bundled,
  };
  const instanceConfig = selectedInstance ? rootConfig.$instances?.[selectedInstance] : undefined;

  const areDevToolsEnabled = getDevTools();

  const { t } = useTranslation();

  const onConfigChange = (key: string, value: any) => {
    // intances `enabled` always inherit from widget root config
    if (selectedInstance && key !== "enabled") {
      patchInstance(widgetId, selectedInstance, { [key]: value });
      return;
    }

    patchWidget(widgetId, { [key]: value });
  };

  const instances = Object.keys(rootConfig.$instances || {}).map((instanceId) => ({
    label: `${t("widget.instance")} ${instanceId.slice(0, 6)}`,
    value: instanceId,
  }));

  const fullConfigPatch = {
    ...rootConfig,
    ...(instanceConfig || {}),
  };

  if (!widget) {
    return <div>404</div>;
  }

  const widgetPlugins = widget.plugins;

  return (
    <>
      <Note>
        <ResourceDescription text={widget.metadata.description} />
      </Note>

      {widgetPlugins.length > 0 && (
        <SettingsGroup>
          <SettingsSubGroup label={t("resources.included_plugins")}>
            <div className={cs.tags}>
              {widgetPlugins.map((plugin) => (
                <div key={plugin.id} className={cs.tag}>
                  <ResourceText text={plugin.metadata.displayName} />
                </div>
              ))}
            </div>
          </SettingsSubGroup>
        </SettingsGroup>
      )}

      <SettingsGroup>
        <SettingsOption>
          <b>{t("widget.enable")}</b>
          <Switch
            checked={fullConfigPatch.enabled}
            onChange={(value) => {
              onConfigChange("enabled", value);
            }}
          />
        </SettingsOption>
        <SettingsOption>
          <b>{t("reset_configurations")}</b>
          <Button onClick={() => resetWidgetConfig(widgetId)}>
            <Icon iconName="RiResetLeftLine" />
          </Button>
        </SettingsOption>
      </SettingsGroup>

      {widget.instances === "Multiple" && (
        <SettingsGroup>
          <SettingsOption>
            <b>{t("widget.default_instance")}</b>
            <span style={{ color: "var(--color-text-3)" }}>
              {t("widget.default_instance_hint")}
            </span>
          </SettingsOption>
          <SettingsOption>
            <b>{t("widget.extra_instances")}</b>
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
        widgetId={widgetId}
        definitions={widget.settings}
        values={fullConfigPatch}
        onConfigChange={onConfigChange}
      />

      {children}

      {areDevToolsEnabled && (
        <SettingsGroup>
          <SettingsSubGroup label={<b>Raw Config</b>}>
            <pre>{JSON.stringify(rootConfig, null, 2)}</pre>
          </SettingsSubGroup>
        </SettingsGroup>
      )}
    </>
  );
}

interface MonitorProps {
  widgetId: WidgetId;
  monitorId: string;
}

function WidgetConfigOnMonitor({ widgetId, monitorId }: MonitorProps) {
  const widget = widgets.value.find((t) => t.id === widgetId);

  const rootConfig = getWidgetConfig(widgetId) || {
    enabled: widget?.loader !== "Legacy" && !!widget?.metadata.bundled,
  };
  const monitorConfig = getMonitorWidgetConfig(monitorId, widgetId) || { enabled: true };

  const areDevToolsEnabled = getDevTools();

  const { t } = useTranslation();

  const onConfigChange = (key: string, value: any) => {
    patchMonitor(monitorId, widgetId, { [key]: value });
  };

  const fullConfigPatch = {
    ...rootConfig,
    ...monitorConfig,
  };

  if (!widget) {
    return <div>404</div>;
  }

  const showToggleEnabled = widget.instances === "ReplicaByMonitor";

  return (
    <>
      <Note>
        <ResourceDescription text={widget.metadata.description} />
      </Note>

      {showToggleEnabled && (
        <SettingsGroup>
          <SettingsOption>
            <b>{t("widget.enable_for_monitor")}</b>
            <Switch
              checked={fullConfigPatch.enabled}
              onChange={(value) => {
                onConfigChange("enabled", value);
              }}
            />
          </SettingsOption>
          <SettingsOption>
            <b>{t("reset_configurations")}</b>
            <Button onClick={() => resetWidgetMonitorConfig(monitorId, widgetId)}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          </SettingsOption>
        </SettingsGroup>
      )}

      <RenderBySettingsDeclaration
        widgetId={widgetId}
        definitions={widget.settings}
        values={fullConfigPatch}
        onConfigChange={onConfigChange}
        monitorId={monitorId}
      />

      {areDevToolsEnabled && (
        <SettingsGroup>
          <SettingsSubGroup label={<b>Root Config</b>}>
            <pre>{JSON.stringify(rootConfig, null, 2)}</pre>
          </SettingsSubGroup>

          <SettingsSubGroup label={<b>Monitor Patch</b>}>
            <pre>{monitorConfig ? JSON.stringify(monitorConfig, null, 2) : "Inherited"}</pre>
          </SettingsSubGroup>
        </SettingsGroup>
      )}
    </>
  );
}

export function WidgetView() {
  const [searchParams] = useSearchParams();
  const widgetId = searchParams.get("id") as WidgetId;

  if (widgetId === "@seelen/weg") {
    return (
      <WidgetConfiguration widgetId={widgetId}>
        <SeelenWegSettings />
      </WidgetConfiguration>
    );
  }

  if (widgetId === "@seelen/window-manager") {
    return (
      <WidgetConfiguration widgetId={widgetId}>
        <WindowManagerSettings />
      </WidgetConfiguration>
    );
  }

  if (widgetId === "@seelen/fancy-toolbar") {
    return (
      <WidgetConfiguration widgetId={widgetId}>
        <FancyToolbarSettings />
      </WidgetConfiguration>
    );
  }

  if (widgetId === "@seelen/wallpaper-manager") {
    return (
      <WidgetConfiguration widgetId={widgetId}>
        <WallSettings />
      </WidgetConfiguration>
    );
  }

  return <WidgetConfiguration widgetId={widgetId} />;
}
