import { SeelenWallWidgetId } from "@seelen-ui/lib";
import type { WidgetId } from "@seelen-ui/lib/types";
import { Switch } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { useParams } from "react-router";

import { RootActions } from "../../shared/store/app/reducer.ts";
import { RootSelectors } from "../../shared/store/app/selectors.ts";

import type { RootState } from "../../shared/store/domain.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { WallpaperList } from "../../Wall/WallpaperList.tsx";
import { RenderBySettingsDeclaration } from "./ConfigRenderer.tsx";
import { WidgetInstanceSelector } from "./InstanceSelector.tsx";

const selectMonitorWidgetConfig = (id: WidgetId, monitorId?: string) => (state: RootState) => {
  if (!monitorId) {
    return undefined;
  }
  return state.monitorsV3[monitorId]?.byWidget[id];
};

const selectWidgetConfig = (id: WidgetId) => (state: RootState) => {
  return state.byWidget[id];
};

const selectWidgetDeclaration = (id: WidgetId) => (state: RootState) => {
  return state.widgets.find((t) => t.id === id);
};

export function WidgetConfiguration({
  widgetId,
  monitorId,
}: {
  widgetId: WidgetId;
  monitorId?: string;
}) {
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null);

  const widget = useSelector(selectWidgetDeclaration(widgetId));
  const rootConfig = useSelector(selectWidgetConfig(widgetId)) ||
    { enabled: true };
  const monitorConfig = useSelector(
    selectMonitorWidgetConfig(widgetId, monitorId),
  );
  const areDevToolsEnabled = useSelector(RootSelectors.devTools);

  const { t } = useTranslation();
  const d = useDispatch();

  if (!widget) {
    return <div>wow 404 !?</div>;
  }

  const onConfigChange = (key: string, value: any) => {
    if (monitorId) {
      d(
        RootActions.patchWidgetMonitorConfig({
          monitorId,
          widgetId,
          config: { [key]: value },
        }),
      );
      return;
    }

    // intances `enabled` always inherit from widget root config
    if (selectedInstance && key !== "enabled") {
      d(
        RootActions.patchWidgetInstanceConfig({
          widgetId,
          instanceId: selectedInstance,
          config: { [key]: value },
        }),
      );
      return;
    }

    d(RootActions.patchWidgetConfig({ widgetId, config: { [key]: value } }));
  };

  const instances = Object.keys(rootConfig.$instances || {}).map((
    instanceId,
  ) => ({
    label: `Instance ${instanceId.slice(0, 6)}`,
    value: instanceId,
  }));

  const instanceConfig = selectedInstance ? rootConfig.$instances?.[selectedInstance] : undefined;
  const config = {
    ...rootConfig,
    ...(instanceConfig || {}),
    ...(monitorConfig || {}),
  };

  const showToggleEnabled = !monitorId ||
    widget.instances === "ReplicaByMonitor";

  return (
    <>
      {showToggleEnabled && (
        <SettingsGroup>
          <SettingsOption>
            <b>
              {monitorId ? t("widget.enable_for_monitor") : t("widget.enable")}
            </b>
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

      {/* special case */}
      {widgetId === SeelenWallWidgetId && <WallpaperList monitorId={monitorId} />}

      {areDevToolsEnabled && (
        <SettingsGroup>
          <SettingsSubGroup label={<b>Raw Config</b>}>
            <pre>{JSON.stringify(rootConfig, null, 2)}</pre>
          </SettingsSubGroup>
          {!!monitorId && (
            <SettingsSubGroup label={<b>Raw Monitor Patch</b>}>
              <pre>{monitorConfig ? JSON.stringify(monitorConfig, null, 2) : 'Inherited'}</pre>
            </SettingsSubGroup>
          )}
        </SettingsGroup>
      )}
    </>
  );
}

export function WidgetView() {
  const { username, resourceName } = useParams<"username" | "resourceName">();
  const widgetId = `@${username}/${resourceName}` as WidgetId;
  return <WidgetConfiguration widgetId={widgetId} />;
}
