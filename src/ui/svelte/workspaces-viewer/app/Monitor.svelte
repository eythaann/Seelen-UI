<script lang="ts">
  import Window from "./Window.svelte";
  import Workspace from "./Workspace.svelte";
  import { state as store } from "../state.svelte";
  import type { PhysicalMonitor } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { Wallpaper } from "libs/ui/svelte/components/Wallpaper";
  import { DragDropProvider, DragOverlay } from "@dnd-kit/svelte";
  import { DND_PLUGINS, DND_SENSORS } from "libs/ui/dnd";

  const { monitor } = $props<{ monitor: PhysicalMonitor }>();

  const width = $derived((monitor.rect.right - monitor.rect.left) / monitor.scaleFactor);
  const height = $derived((monitor.rect.bottom - monitor.rect.top) / monitor.scaleFactor);

  const vdMonitor = $derived(store.workspaces.monitors[monitor.id]);
  const activeWorkspaceId = $derived(vdMonitor?.active_workspace);

  let viewingWorkspaceId: string | undefined = $state(undefined);

  $effect(() => {
    viewingWorkspaceId = activeWorkspaceId;
  });

  const viewingWorkspace = $derived(vdMonitor?.workspaces.find((w) => w.id === viewingWorkspaceId));

  const activeWallpaper = $derived(store.findWallpaper(viewingWorkspace?.wallpaper));

  async function createWorkspace(e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke(SeelenCommand.CreateWorkspace, { monitorId: monitor.id });
    } catch (error) {
      console.error("Failed to create workspace:", error);
    }
  }
</script>

<DragDropProvider
  plugins={DND_PLUGINS}
  sensors={DND_SENSORS}
  onDragEnd={(event) => {
    const { source, target } = event.operation;
    if (!source || !target) return;
    const hwnd = source.id as number;
    const workspaceId = target.id as string;
    if (viewingWorkspaceId === workspaceId) return;
    invoke(SeelenCommand.MoveWindowToWorkspace, { hwnd, workspaceId });
  }}
>
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

    <div class="slu-std-surface workspaces">
      {#if vdMonitor}
        {#each vdMonitor.workspaces as workspace, index}
          <Workspace
            active={vdMonitor.active_workspace === workspace.id}
            viewing={viewingWorkspaceId === workspace.id}
            {workspace}
            {index}
            onHover={() => {
              viewingWorkspaceId = workspace.id;
            }}
          />
        {/each}
      {/if}

      <button class="slu-std-surface-elevated add-workspace" onclick={createWorkspace}>
        <Icon iconName="IoAdd" />
      </button>
    </div>

    {#if viewingWorkspace}
      <div class="windows">
        {#each viewingWorkspace.windows as hwnd}
          <Window {hwnd} />
        {/each}
      </div>
    {/if}

    <DragOverlay dropAnimation={null}>
      {#snippet children(source)}
        <Window hwnd={source.id as number} overlay />
      {/snippet}
    </DragOverlay>
  </div>
</DragDropProvider>
