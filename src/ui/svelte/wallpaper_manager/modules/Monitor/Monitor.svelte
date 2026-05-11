<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { WallpaperKind, type PhysicalMonitor } from "@seelen-ui/lib/types";
  import { Wallpaper } from "libs/ui/svelte/components/Wallpaper";
  import { gState } from "../../state.svelte.ts";
  import { t } from "../../i18n/index.ts";
  import { extractAccentColorFromSrc } from "../accentExtractor.ts";
  import { convertFileSrc } from "@tauri-apps/api/core";

  let { monitor, extended = false }: { monitor: PhysicalMonitor; extended?: boolean } = $props();

  let currentWasLoaded = $state(false);

  const player = $derived(gState.players.find((p) => p.default));

  const wallpaperId = $derived.by(() => {
    const monitorData = gState.virtualDesktops.monitors[monitor.id];
    if (!monitorData) {
      return null;
    }

    const activeWorkspace = monitorData.workspaces.find(
      (ws) => ws.id === monitorData.active_workspace,
    );
    return activeWorkspace?.wallpaper || null;
  });

  // Ping-pong: two slots alternate roles on each wallpaper change.
  // The incoming wallpaper always mounts in the idle slot (never the visible one),
  // so the currently-displayed wallpaper is never remounted — eliminating flicker.
  // After the transition the outgoing slot is fully unmounted to free resources;
  // on the next change it will remount fresh in the idle position, which is fine.
  let activeSlot = $state<"a" | "b">("a");

  // svelte-ignore state_referenced_locally
  let slotAWallpaperId = $state<string | null>(wallpaperId);
  let slotBWallpaperId = $state<string | null>(null);
  let slotAOut = $state(false);
  let slotBOut = $state(false);
  let slotAMounted = $state(true);
  let slotBMounted = $state(false);

  const slotAWallpaper = $derived(gState.findWallpaper(slotAWallpaperId));
  const slotBWallpaper = $derived(gState.findWallpaper(slotBWallpaperId));

  // svelte-ignore state_referenced_locally
  let lastActiveRef: { value: string | null } = { value: wallpaperId };

  $effect(() => {
    if (lastActiveRef.value === wallpaperId) return;
    lastActiveRef.value = wallpaperId;
    currentWasLoaded = false;

    if (activeSlot === "a") {
      slotBWallpaperId = wallpaperId;
      slotBMounted = true;
      slotBOut = false;
      slotAOut = true;
      activeSlot = "b";
    } else {
      slotAWallpaperId = wallpaperId;
      slotAMounted = true;
      slotAOut = false;
      slotBOut = true;
      activeSlot = "a";
    }
  });

  // Unmount the outgoing slot 1s after the new wallpaper finishes loading.
  $effect(() => {
    if (!currentWasLoaded) return;
    const timeoutId = setTimeout(() => {
      if (activeSlot === "b") {
        slotAMounted = false;
      } else {
        slotBMounted = false;
      }
    }, 1000);
    return () => clearTimeout(timeoutId);
  });

  const accentFilename = $derived.by(() => {
    const wallpaper = activeSlot === "a" ? slotAWallpaper : slotBWallpaper;
    if (!wallpaper) return null;

    if (wallpaper.type === WallpaperKind.MediaPlayer) {
      return player?.thumbnail;
    }

    if (wallpaper.type === WallpaperKind.Video && wallpaper.thumbnailFilename) {
      return wallpaper.metadata.path + "\\" + wallpaper.thumbnailFilename;
    }

    if (wallpaper.type === WallpaperKind.Image && wallpaper.filename) {
      return wallpaper.metadata.path + "\\" + wallpaper.filename;
    }

    return null;
  });

  $effect(() => {
    if (!monitor.isPrimary || !accentFilename || !gState.settings.useAccentColor) return;
    extractAccentColorFromSrc(convertFileSrc(accentFilename)).then((color) => {
      if (color) {
        invoke(SeelenCommand.SystemSetAccentColor, { color });
      }
    });
  });

  const left = $derived(extended ? "0" : `${monitor.rect.left / globalThis.devicePixelRatio}px`);
  const top = $derived(extended ? "0" : `${monitor.rect.top / globalThis.devicePixelRatio}px`);
  const width = $derived(
    extended
      ? "100%"
      : `${(monitor.rect.right - monitor.rect.left) / globalThis.devicePixelRatio}px`,
  );
  const height = $derived(
    extended
      ? "100%"
      : `${(monitor.rect.bottom - monitor.rect.top) / globalThis.devicePixelRatio}px`,
  );
</script>

<div class="monitor" style:position="fixed" style:left style:top style:width style:height>
  {#if slotAMounted}
    <div class="slot" style:z-index={activeSlot === "a" ? 2 : 1}>
      <Wallpaper
        definition={slotAWallpaper}
        config={slotAWallpaper ? gState.settings.byWallpaper[slotAWallpaper.id] : undefined}
        onLoad={activeSlot === "a" ? () => (currentWasLoaded = true) : undefined}
        paused={slotAOut || gState.isPaused(monitor.id)}
        muted={slotAOut || gState.muted || !monitor.isPrimary}
        out={slotAOut}
        pausedMessage={activeSlot === "a" && gState.performanceMode !== "Disabled"
          ? $t("paused_by_performance_mode")
          : undefined}
      />
    </div>
  {/if}

  {#if slotBMounted}
    <div class="slot" style:z-index={activeSlot === "b" ? 2 : 1}>
      <Wallpaper
        definition={slotBWallpaper}
        config={slotBWallpaper ? gState.settings.byWallpaper[slotBWallpaper.id] : undefined}
        onLoad={activeSlot === "b" ? () => (currentWasLoaded = true) : undefined}
        paused={slotBOut || gState.isPaused(monitor.id)}
        muted={slotBOut || gState.muted || !monitor.isPrimary}
        out={slotBOut}
        pausedMessage={activeSlot === "b" && gState.performanceMode !== "Disabled"
          ? $t("paused_by_performance_mode")
          : undefined}
      />
    </div>
  {/if}
</div>

<style>
  .slot {
    position: absolute;
    inset: 0;
  }
</style>
