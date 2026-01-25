import { settings } from "../../../state/mod";
import type { WidgetId } from "@seelen-ui/lib/types";

/**
 * Gets the configuration for a specific widget
 */
export function getWidgetConfig(widgetId: WidgetId) {
  return settings.value.byWidget[widgetId];
}

/**
 * Patches the configuration for a specific widget
 */
export function patchWidgetConfig(widgetId: WidgetId, config: Record<string, unknown>) {
  const currentWidget = settings.value.byWidget[widgetId] || { enabled: true };

  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      [widgetId]: {
        ...currentWidget,
        ...config,
      },
    },
  };
}

/**
 * Gets the configuration for a specific widget instance
 */
export function getWidgetInstanceConfig(widgetId: WidgetId, instanceId: string) {
  return settings.value.byWidget[widgetId]?.$instances?.[instanceId];
}

/**
 * Patches the configuration for a specific widget instance
 */
export function patchWidgetInstanceConfig(
  widgetId: WidgetId,
  instanceId: string,
  config: Record<string, any>,
) {
  const currentWidget = settings.value.byWidget[widgetId] || { enabled: true };
  const instances = currentWidget.$instances || {};
  const currentInstance = instances[instanceId] || {};

  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      [widgetId]: {
        ...currentWidget,
        $instances: {
          ...instances,
          [instanceId]: {
            ...currentInstance,
            ...config,
          },
        },
      },
    },
  };
}

/**
 * Removes a widget instance
 */
export function removeWidgetInstance(widgetId: WidgetId, instanceId: string) {
  const currentWidget = settings.value.byWidget[widgetId];
  if (!currentWidget) {
    return;
  }

  const instances = currentWidget.$instances || {};
  const { [instanceId]: _, ...remainingInstances } = instances;

  const newWidget = { ...currentWidget };
  if (Object.keys(remainingInstances).length === 0) {
    delete newWidget.$instances;
  } else {
    newWidget.$instances = remainingInstances;
  }

  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      [widgetId]: newWidget,
    },
  };
}

/**
 * Gets the monitor-specific configuration for a widget
 */
export function getMonitorWidgetConfig(monitorId: string, widgetId: WidgetId) {
  return settings.value.monitorsV3[monitorId]?.byWidget[widgetId];
}

/**
 * Patches the monitor-specific configuration for a widget
 */
export function patchWidgetMonitorConfig(
  monitorId: string,
  widgetId: WidgetId,
  config: Record<string, any>,
) {
  const monitor = settings.value.monitorsV3[monitorId];
  if (!monitor) {
    return;
  }

  const currentWidgetConfig = monitor.byWidget[widgetId] || { enabled: true };

  settings.value = {
    ...settings.value,
    monitorsV3: {
      ...settings.value.monitorsV3,
      [monitorId]: {
        ...monitor,
        byWidget: {
          ...monitor.byWidget,
          [widgetId]: {
            ...currentWidgetConfig,
            ...config,
          },
        },
      },
    },
  };
}
