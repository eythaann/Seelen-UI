import { SeelenWallWidgetId } from "@seelen-ui/lib";
import type { PhysicalMonitor, Widget } from "@seelen-ui/lib/types";
import { ResourceText } from "@shared/components/ResourceText";
import { useSelector } from "react-redux";

import { newSelectors } from "../../shared/store/app/reducer.ts";

import { Monitor } from "../../../components/monitor/index.tsx";
import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";
import { WidgetSettingsModal } from "./WidgetSettingsModal.tsx";
import cs from "./index.module.css";

interface MonitorConfigProps {
  device: PhysicalMonitor;
}

export function MonitorConfig({ device }: MonitorConfigProps) {
  const widgets = useSelector(newSelectors.widgets);

  return (
    <SettingsGroup>
      <div className={cs.itemContainer}>
        <div className={cs.itemLeft}>
          <div className={cs.label}>{device.name}</div>
          <Monitor
            monitorId={device.id}
            width={device.rect.right - device.rect.left}
            height={device.rect.bottom - device.rect.top}
          />
        </div>
        <div className={cs.itemRight}>
          <SettingsGroup>
            <SettingsOption>
              <b>Resolution</b>
              <div>
                {device.rect.right - device.rect.left} x {device.rect.bottom - device.rect.top}
              </div>
            </SettingsOption>
          </SettingsGroup>

          <SettingsGroup>
            {widgets.filter(isConfigurableByMonitor).map((widget) => {
              return (
                <SettingsOption key={widget.id}>
                  <ResourceText text={widget.metadata.displayName} />
                  <WidgetSettingsModal
                    widgetId={widget.id}
                    monitorId={device.id}
                    title={
                      <>
                        {device.name}
                        {" / "}
                        <ResourceText text={widget.metadata.displayName} />
                      </>
                    }
                  />
                </SettingsOption>
              );
            })}
          </SettingsGroup>
        </div>
      </div>
    </SettingsGroup>
  );
}

export function SettingsByMonitor() {
  const devices = useSelector(newSelectors.connectedMonitors);
  const settingsByMonitor = useSelector(newSelectors.monitorsV3);

  return (
    <>
      {devices.map((device) => {
        let monitor = settingsByMonitor[device.id];
        if (!monitor) {
          console.warn(`Monitor settings not initialized ${device.id}`);
          return null;
        }
        return <MonitorConfig key={device.id} device={device} />;
      })}
    </>
  );
}

function isConfigurableByMonitor(widget: Widget) {
  if (widget.instances === "ReplicaByMonitor") {
    return true;
  }

  // special case
  if (widget.id === SeelenWallWidgetId) {
    return true;
  }

  for (const { group } of widget.settings) {
    for (const entry of group) {
      const stack = [entry];

      while (stack.length > 0) {
        const entry = stack.pop()!;
        if (entry.config.allowSetByMonitor) {
          return true;
        }
        stack.push(...entry.children);
      }
    }
  }

  return false;
}
