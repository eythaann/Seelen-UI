import { batch, useComputed, useSignal, useSignalEffect } from "@preact/signals";
import type { PhysicalMonitor, WallpaperId } from "@seelen-ui/lib/types";
import { Wallpaper as WallpaperComponent } from "libs/ui/react/components/Wallpaper/index.tsx";
import { useTranslation } from "react-i18next";

import { $muted, $paused, $performance_mode, $settings, $virtualDesktops, $wallpapers } from "../shared/state.ts";
import { $relativeMonitors } from "./derived.ts";

export function MonitorContainers() {
  return $relativeMonitors.value.map((monitor) => {
    return <Monitor key={monitor.id} monitor={monitor} />;
  });
}

/*
 * cases:
 * 1. wallpaper from active workspace:
 *    - show wallpaper from the currently active workspace on this monitor
 * 2. transitions when workspace changes:
 *    - old wallpaper will persist for 1 second during transition
 * 3. performance mode will disable video wallpapers
 */
function Monitor({ monitor }: { monitor: PhysicalMonitor }) {
  const { t } = useTranslation();
  const $render_old = useSignal(false);
  const $current_was_loaded = useSignal(false);

  // unrender old wallpaper after 1s
  useSignalEffect(() => {
    if (!$render_old.value || !$current_was_loaded.value) return;
    // start unrender timeout only after success load of new wallpaper
    let timeoutId = setTimeout(() => {
      $render_old.value = false;
    }, 1000);
    return () => clearTimeout(timeoutId);
  });

  // Get wallpaper from active workspace for this monitor
  const $active_wallpaper_id = useComputed(() => {
    const monitorData = $virtualDesktops.value.monitors[monitor.id];
    if (!monitorData) return null;

    const activeWorkspace = monitorData.workspaces.find(
      (ws) => ws.id === monitorData.active_workspace,
    );
    return activeWorkspace?.wallpaper || null;
  });

  const $old_id = useSignal<WallpaperId | null>(null);
  const $current_id = useSignal<WallpaperId | null>($active_wallpaper_id.value);

  // Watch for changes in active wallpaper and apply transition
  useSignalEffect(() => {
    const newWallpaperId = $active_wallpaper_id.value;
    if (newWallpaperId !== $current_id.value) {
      batch(() => {
        $old_id.value = $current_id.value;
        $render_old.value = true;
        $current_id.value = newWallpaperId;
        $current_was_loaded.value = false;
      });
    }
  });

  const oldWallpaper = $wallpapers.value.find((wallpaper) => wallpaper.id === $old_id.value);
  const wallpaper = $wallpapers.value.find((wallpaper) => wallpaper.id === $current_id.value);

  if ($old_id.value && !oldWallpaper) {
    console.error("Old wallpaper not found (maybe removed?)", $old_id.value);
  }
  if ($current_id.value && !wallpaper) {
    console.error("Wallpaper not found (maybe removed?)", $current_id.value);
  }

  return (
    <div
      className="monitor"
      style={{
        position: "fixed",
        left: monitor.rect.left / globalThis.devicePixelRatio,
        top: monitor.rect.top / globalThis.devicePixelRatio,
        width: (monitor.rect.right - monitor.rect.left) /
          globalThis.devicePixelRatio,
        height: (monitor.rect.bottom - monitor.rect.top) /
          globalThis.devicePixelRatio,
      }}
    >
      {[
        $render_old.value && (
          <WallpaperComponent
            key={oldWallpaper?.id || "themed"}
            definition={oldWallpaper}
            config={oldWallpaper &&
              $settings.value.byWallpaper[oldWallpaper.id]}
            paused // inmediately pause exiting wallpaper, to avoid gpu usage.
            out={$current_was_loaded.value}
          />
        ),
        <WallpaperComponent
          key={wallpaper?.id || "themed"}
          definition={wallpaper}
          config={wallpaper && $settings.value.byWallpaper[wallpaper.id]}
          onLoad={() => ($current_was_loaded.value = true)}
          paused={$paused.value}
          muted={$muted.value || !monitor.isPrimary}
          pausedMessage={$performance_mode.value !== "Disabled" ? t("paused_by_performance_mode") : undefined}
        />,
      ]}
    </div>
  );
}
