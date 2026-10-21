import { DEFAULT_SETTINGS, settings } from "../../../state/mod";
import type { GenericWidgetSettings, WidgetId } from "@seelen-ui/lib/types";
import { widgets } from "../../../state/resources";

/**
 * Gets the configuration for a specific widget
 */
export function getWidgetConfig(widgetId: WidgetId) {
  return settings.value.byWidget[widgetId];
}

export function isWidgetEnabled(widgetId: WidgetId) {
  const widget = widgets.value.find((t) => t.id === widgetId);
  const config = settings.value.byWidget[widgetId];
  if (config) {
    return config.enabled;
  }
  return !!widget?.metadata.bundled;
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
 * Resets all user-set config values for a widget, preserving only `enabled` and `$instances`.
 */
export function resetWidgetConfig(widgetId: WidgetId) {
  const config = settings.value.byWidget[widgetId];
  if (!config) {
    return;
  }

  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      [widgetId]: {
        ...(DEFAULT_SETTINGS.byWidget[widgetId] ?? {}),
        // Keep enabled state
        enabled: config.enabled,
      },
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

/**
 * Resets the monitor-specific config values for a widget (including its workspace overrides on
 * that monitor), preserving only `enabled`. Root config is untouched, so the monitor inherits it again.
 */
export function resetWidgetMonitorConfig(monitorId: string, widgetId: WidgetId) {
  const monitor = settings.value.monitorsV3[monitorId];
  if (!monitor) {
    return;
  }

  const { [widgetId]: current, ...byWidget } = monitor.byWidget;
  if (current) {
    byWidget[widgetId] = { enabled: current.enabled };
  }

  const byWorkspace = Object.fromEntries(
    Object.entries(monitor.byWorkspace).map(([workspaceId, workspace]) => {
      const { [widgetId]: _, ...workspaceByWidget } = workspace.byWidget;
      return [workspaceId, { ...workspace, byWidget: workspaceByWidget }];
    }),
  );

  settings.value = {
    ...settings.value,
    monitorsV3: {
      ...settings.value.monitorsV3,
      [monitorId]: { ...monitor, byWidget, byWorkspace },
    },
  };
}

/**
 * Gets the workspace-specific configuration for a widget
 */
export function getWorkspaceWidgetConfig(monitorId: string, workspaceId: string, widgetId: WidgetId) {
  return settings.value.monitorsV3[monitorId]?.byWorkspace[workspaceId]?.byWidget[widgetId];
}

/**
 * Replaces the workspace-specific configuration for a widget, removing it when empty
 */
function setWidgetWorkspaceConfig(
  monitorId: string,
  workspaceId: string,
  widgetId: WidgetId,
  config: GenericWidgetSettings | undefined,
) {
  const monitor = settings.value.monitorsV3[monitorId];
  if (!monitor) {
    return;
  }

  const workspace = monitor.byWorkspace[workspaceId] || { byWidget: {}, wallpaperCollection: null };
  const { [widgetId]: _, ...byWidget } = workspace.byWidget;
  if (config) {
    byWidget[widgetId] = config;
  }

  settings.value = {
    ...settings.value,
    monitorsV3: {
      ...settings.value.monitorsV3,
      [monitorId]: {
        ...monitor,
        byWorkspace: {
          ...monitor.byWorkspace,
          [workspaceId]: { ...workspace, byWidget },
        },
      },
    },
  };
}

/**
 * Patches the workspace-specific configuration for a widget
 */
export function patchWidgetWorkspaceConfig(
  monitorId: string,
  workspaceId: string,
  widgetId: WidgetId,
  config: Record<string, any>,
) {
  const current = getWorkspaceWidgetConfig(monitorId, workspaceId, widgetId) || { enabled: true };
  setWidgetWorkspaceConfig(monitorId, workspaceId, widgetId, { ...current, ...config });
}

/**
 * Removes a key from the workspace-specific configuration of a widget,
 * so the workspace inherits it again from the monitor.
 */
export function removeWidgetWorkspaceConfigKey(
  monitorId: string,
  workspaceId: string,
  widgetId: WidgetId,
  key: string,
) {
  const current = getWorkspaceWidgetConfig(monitorId, workspaceId, widgetId);
  if (!current) {
    return;
  }

  const { [key]: _, ...remaining } = current;
  // `enabled` is not overridable by workspace, so only it remaining means there's nothing left
  const isEmpty = Object.keys(remaining).every((k) => k === "enabled");
  setWidgetWorkspaceConfig(
    monitorId,
    workspaceId,
    widgetId,
    isEmpty ? undefined : (remaining as GenericWidgetSettings),
  );
}
