import { batch, computed, useSignal, useSignalEffect } from '@preact/signals';
import { PhysicalMonitor, WallpaperId } from '@seelen-ui/lib/types';

import { ThemedWallpaper, Wallpaper as WallpaperComponent } from '../wallpaper/infra';

import { $active_wallpapers, $monitors, $settings } from '../shared/state';

const $relativeMonitors = computed(() => {
  const lower = { x: 0, y: 0 };
  for (const monitor of $monitors.value) {
    if (monitor.rect.left < lower.x) {
      lower.x = monitor.rect.left;
    }
    if (monitor.rect.top < lower.y) {
      lower.y = monitor.rect.top;
    }
  }

  return $monitors.value.map((monitor) => {
    return {
      ...monitor,
      rect: {
        ...monitor.rect,
        left: monitor.rect.left - lower.x,
        top: monitor.rect.top - lower.y,
        right: monitor.rect.right - lower.x,
        bottom: monitor.rect.bottom - lower.y,
      },
    };
  });
});

export function MonitorContainers() {
  console.log($relativeMonitors.value);
  return $relativeMonitors.value.map((monitor) => {
    return <Monitor key={monitor.id} monitor={monitor} />;
  });
}

function Monitor({ monitor }: { monitor: PhysicalMonitor }) {
  const renderOld = useSignal(false);

  const oldId = useSignal<WallpaperId | null>(null);
  const currentId = useSignal<WallpaperId | null>($active_wallpapers.value.at(0)?.id || null);

  useSignalEffect(() => {
    if ($active_wallpapers.value.length < 2) {
      batch(() => {
        renderOld.value = false;
        oldId.value = null;
        currentId.value = $active_wallpapers.value.at(0)?.id || null;
      });
      return;
    }

    let intervalId = window.setInterval(() => {
      const index = $active_wallpapers.value.findIndex(
        (wallpaper) => wallpaper.id === currentId.value,
      );
      const nextIndex = (index + 1) % $active_wallpapers.value.length;
      batch(() => {
        renderOld.value = true;
        oldId.value = currentId.value;
        currentId.value = $active_wallpapers.value[nextIndex]?.id || null;
      });
    }, $settings.value.interval * 1000);
    return () => clearInterval(intervalId);
  });

  // unrender old wallpaper after 1s
  useSignalEffect(() => {
    if (!renderOld.value) return;
    let timeoutId = setTimeout(() => {
      renderOld.value = false;
    }, 1000);
    return () => clearTimeout(timeoutId);
  });

  const oldWallpaper = $active_wallpapers.value.find((wallpaper) => wallpaper.id === oldId.value);
  const wallpaper = $active_wallpapers.value.find((wallpaper) => wallpaper.id === currentId.value);
  return (
    <div
      className="monitor"
      style={{
        position: 'fixed',
        left: monitor.rect.left / window.devicePixelRatio,
        top: monitor.rect.top / window.devicePixelRatio,
        width: (monitor.rect.right - monitor.rect.left) / window.devicePixelRatio,
        height: (monitor.rect.bottom - monitor.rect.top) / window.devicePixelRatio,
      }}
    >
      {[
        wallpaper ? (
          <WallpaperComponent
            key={wallpaper.id}
            path={wallpaper.metadata.path + '/' + wallpaper.filename}
          />
        ) : (
          <ThemedWallpaper key={renderOld.value ? 'new' : 'current'} />
        ),
        renderOld.value &&
          (oldWallpaper ? (
            <WallpaperComponent
              key={oldWallpaper.id}
              path={oldWallpaper.metadata.path + '/' + oldWallpaper.filename}
              out
            />
          ) : (
            <ThemedWallpaper key="current" out />
          )),
      ]}
      {$settings.value.withOverlay && (
        <div
          className="monitor-overlay"
          style={{
            mixBlendMode: $settings.value.overlayMixBlendMode,
            backgroundColor: $settings.value.overlayColor,
          }}
        />
      )}
    </div>
  );
}
