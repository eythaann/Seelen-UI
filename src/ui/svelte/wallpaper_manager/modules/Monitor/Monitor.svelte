<script lang="ts">
  import type { PhysicalMonitor } from "@seelen-ui/lib/types";
  import { Wallpaper } from "libs/ui/svelte/components/Wallpaper";
  import { gState } from "../../state.svelte.ts";
  import { t } from "../../i18n/index.ts";

  let { monitor, extended = false }: { monitor: PhysicalMonitor; extended?: boolean } = $props();

  let renderOld = $state(false);
  let currentWasLoaded = $state(false);

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

  let oldId = $state<string | null>(null);
  // svelte-ignore state_referenced_locally
  let lastActiveRef: { value: string | null } = { value: wallpaperId };

  // Watch for active wallpaper changes and trigger transition
  $effect(() => {
    const lastWallpaperId = lastActiveRef.value;
    if (lastWallpaperId !== wallpaperId) {
      lastActiveRef.value = wallpaperId;
      oldId = lastWallpaperId;
      renderOld = true;
      currentWasLoaded = false;
    }
  });

  // Unrender old wallpaper after 1s once current has loaded
  $effect(() => {
    if (!renderOld || !currentWasLoaded) return;
    const timeoutId = setTimeout(() => {
      renderOld = false;
    }, 1000);
    return () => clearTimeout(timeoutId);
  });

  const oldWallpaper = $derived(gState.findWallpaper(oldId));
  const wallpaper = $derived(gState.findWallpaper(wallpaperId));

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
  {#if renderOld}
    {#key oldWallpaper?.id ?? "themed"}
      <Wallpaper
        definition={oldWallpaper}
        config={oldWallpaper ? gState.settings.byWallpaper[oldWallpaper.id] : undefined}
        paused
        out={currentWasLoaded}
      />
    {/key}
  {/if}

  {#key wallpaper?.id ?? "themed"}
    <Wallpaper
      definition={wallpaper}
      config={wallpaper ? gState.settings.byWallpaper[wallpaper.id] : undefined}
      onLoad={() => (currentWasLoaded = true)}
      paused={gState.paused}
      muted={gState.muted || !monitor.isPrimary}
      pausedMessage={gState.performanceMode !== "Disabled"
        ? $t("paused_by_performance_mode")
        : undefined}
    />
  {/key}
</div>
