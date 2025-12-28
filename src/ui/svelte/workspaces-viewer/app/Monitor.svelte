<script lang="ts">
  import Window from "./Window.svelte";
  import Workspace from "./Workspace.svelte";
  import { state } from "../state.svelte";
  import type { PhysicalMonitor } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { Wallpaper } from "libs/ui/svelte/components/Wallpaper";

  const props: { monitor: PhysicalMonitor } = $props();
  const { monitor } = props;

  const width = $derived((monitor.rect.right - monitor.rect.left) / monitor.scaleFactor);
  const height = $derived((monitor.rect.bottom - monitor.rect.top) / monitor.scaleFactor);

  const vdMonitor = $derived(state.workspaces.monitors[monitor.id]);
  const activeWorkspace = $derived(
    vdMonitor?.workspaces.find((w) => w.id === vdMonitor?.active_workspace)
  );

  const activeWallpaper = $derived(state.findWallpaper(activeWorkspace?.wallpaper));

  async function createWorkspace(e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke(SeelenCommand.CreateWorkspace, { monitorId: monitor.id });
    } catch (error) {
      console.error("Failed to create workspace:", error);
    }
  }
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
  <div class="active-wallpaper">
    <Wallpaper definition={activeWallpaper} static muted />
  </div>

  <div class="workspaces">
    {#if vdMonitor}
      {#each vdMonitor.workspaces as workspace, index}
        <Workspace active={vdMonitor.active_workspace === workspace.id} {workspace} {index} />
      {/each}
    {/if}

    <button class="add-workspace" onclick={createWorkspace}>
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
