import { batch, useComputed, useSignal, useSignalEffect } from "@preact/signals";
import type { PhysicalMonitor, WallpaperId } from "@seelen-ui/lib/types";
import { Wallpaper as WallpaperComponent } from "@shared/components/Wallpaper";
import { useTranslation } from "react-i18next";

import { $muted, $paused, $performance_mode, $settings, $wallpapers } from "../shared/state.ts";
import { $get_active_wallpapers, $relativeMonitors } from "./derived.ts";

export function MonitorContainers() {
  return $relativeMonitors.value.map((monitor) => {
    return <Monitor key={monitor.id} monitor={monitor} />;
  });
}

/*
 * cases:
 * 1. only one wallpaper:
 *    - no transitions, no old wallpaper, show themed as fallback
 * 2. two or more wallpapers:
 *    - interval change between wallpapers
 *    - in case of error, old wallpaper will persist for 1 second
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

  const $active_wallpapers = useComputed(() => $get_active_wallpapers(monitor.id));
  function $get_initial_wall_id() {
    if ($settings.value.randomize) {
      const idx = Math.floor(Math.random() * $active_wallpapers.value.length);
      return $active_wallpapers.value[idx]?.id || null;
    } else {
      return $active_wallpapers.value.at(0)?.id || null;
    }
  }

  const $old_id = useSignal<WallpaperId | null>(null);
  const $current_id = useSignal<WallpaperId | null>($get_initial_wall_id());

  function changeWallpaper(newId: WallpaperId) {
    batch(() => {
      $old_id.value = $current_id.value;
      $render_old.value = true;
      $current_id.value = newId;
      $current_was_loaded.value = false;
    });
  }

  function changeToNext() {
    let currentIdx = $current_id.value ? $active_wallpapers.value.findIndex((w) => w.id === $current_id.value) : 0;

    // if current was removed from actives
    if (currentIdx === -1) {
      currentIdx = 0;
    }

    let nextIdx = 0;
    if ($settings.value.randomize && $active_wallpapers.value.length > 2) {
      do {
        nextIdx = Math.floor(Math.random() * $active_wallpapers.value.length);
      } while (nextIdx === currentIdx);
    } else {
      // secuential
      nextIdx = (currentIdx + 1) % $active_wallpapers.value.length;
    }

    // if next is same as current (1 wallpaper)
    if (currentIdx === nextIdx) {
      return;
    }

    changeWallpaper($active_wallpapers.value[nextIdx]!.id);
  }

  useSignalEffect(() => {
    if ($active_wallpapers.value.length < 2) {
      batch(() => {
        $render_old.value = false;
        $old_id.value = null;
        $current_id.value = $active_wallpapers.value.at(0)?.id || null;
      });
      return;
    }

    let intervalId = globalThis.setInterval(
      changeToNext,
      $settings.value.interval * 1000,
    );
    return () => clearInterval(intervalId);
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
