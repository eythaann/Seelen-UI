<script lang="ts">
  import { MultimonitorBehaviour } from "@seelen-ui/lib/types";
  import { gState } from "../../state.svelte.ts";
  import Monitor from "./Monitor.svelte";

  const isExtendMode = $derived(
    gState.settings.multimonitorBehaviour === MultimonitorBehaviour.Extend,
  );

  const primaryMonitor = $derived(
    gState.monitors.find((m) => m.isPrimary) ?? gState.monitors[0],
  );

  const player = $derived(gState.players.find((p) => p.default));
</script>

{#if isExtendMode || player}
  {#if primaryMonitor}
    <Monitor monitor={primaryMonitor} extended />
  {/if}
{:else}
  {#each gState.relativeMonitors as monitor (monitor.id)}
    <Monitor {monitor} />
  {/each}
{/if}
