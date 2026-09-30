<script lang="ts">
  import { MultimonitorBehaviour } from "@seelen-ui/lib/types";
  import { gState } from "../../state.svelte.ts";
  import Monitor from "./Monitor.svelte";
  import { monitors } from "libs/ui/svelte/runes/monitors.svelte.ts";

  const isExtendMode = $derived(
    gState.settings.multimonitorBehaviour === MultimonitorBehaviour.Extend,
  );
</script>

{#if isExtendMode}
  {#if monitors.primaryMonitor}
    <Monitor monitor={monitors.primaryMonitor} extended />
  {/if}
{:else}
  {#each monitors.all as monitor (monitor.id)}
    <Monitor {monitor} />
  {/each}
{/if}
