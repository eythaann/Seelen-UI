<script lang="ts">
  import Window from "./Window.svelte";
  import Workspace from "./Workspace.svelte";
  import { state } from "../state.svelte";
  import type { PhysicalMonitor } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";

  const props: { monitor: PhysicalMonitor } = $props();
  const { monitor } = props;

  const width = $derived((monitor.rect.right - monitor.rect.left) / monitor.scaleFactor);
  const height = $derived((monitor.rect.bottom - monitor.rect.top) / monitor.scaleFactor);

  const vdMonitor = $derived(state.workspaces.monitors[monitor.id]);
  const activeWorkspace = $derived(
    vdMonitor?.workspaces.find((w) => w.id === vdMonitor?.active_workspace)
  );
</script>

<div
  class="monitor"
  style:position="fixed"
  style:left={monitor.rect.left + "px"}
  style:top={monitor.rect.top + "px"}
  style:width={width + "px"}
  style:height={height + "px"}
  style:transform={`scale(${monitor.scaleFactor})`}
  style:transform-origin="left top"
>
  <div class="workspaces">
    {#if vdMonitor}
      {#each vdMonitor.workspaces as workspace, index}
        <Workspace active={vdMonitor.active_workspace === workspace.id} {workspace} {index} />
      {/each}
    {/if}

    <button class="add-workspace">
      <Icon iconName="IoAdd" />
    </button>
  </div>

  {#if activeWorkspace}
    <div class="windows">
      {#each activeWorkspace.windows as hwnd}
        <Window {hwnd} />
      {/each}
    </div>
  {/if}
</div>
