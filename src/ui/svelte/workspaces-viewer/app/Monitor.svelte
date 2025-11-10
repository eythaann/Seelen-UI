<script lang="ts">
  import Workspace from "./Workspace.svelte";
  import { state } from "../state.svelte";
  import type { PhysicalMonitor } from "@seelen-ui/lib/types";

  const props: { monitor: PhysicalMonitor } = $props();
  const { monitor } = props;

  const scaleFactor = globalThis.devicePixelRatio;
  const left = $derived(monitor.rect.left / scaleFactor);
  const top = $derived(monitor.rect.top / scaleFactor);
  const width = $derived((monitor.rect.right - monitor.rect.left) / scaleFactor);
  const height = $derived((monitor.rect.bottom - monitor.rect.top) / scaleFactor);

  const vd = $derived(state.workspaces.monitors[monitor.id]);
</script>

<div
  class="monitor"
  style:position="fixed"
  style:left={left + "px"}
  style:top={top + "px"}
  style:width={width + "px"}
  style:height={height + "px"}
>
  <div class="windows"></div>

  <div class="workspaces">
    {#if vd}
      {#each vd.workspaces as workspace, index}
        <Workspace {workspace} {index} />
      {/each}
    {/if}
  </div>
</div>
