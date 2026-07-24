<script lang="ts">
  import type { DesktopWorkspace } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { Wallpaper } from "libs/ui/svelte/components/Wallpaper";
  import { state as store } from "../state.svelte";
  import { createDroppable } from "@dnd-kit/svelte";

  interface Props {
    index: number;
    workspace: DesktopWorkspace;
    active: boolean;
    viewing: boolean;
    onHover: () => void;
  }

  let { workspace, index, active, viewing, onHover }: Props = $props();

  const wallpaper = $derived(store.findWallpaper(workspace.wallpaper));

  let workspaceName = $state("");

  $effect(() => {
    workspaceName = workspace.name || "";
  });

  const droppable = createDroppable({
    get id() {
      return workspace.id;
    },
  });

  async function switchWorkspace() {
    if (active) return;
    await Widget.self.hide();
    await invoke(SeelenCommand.SwitchWorkspace, {
      workspaceId: workspace.id,
    });
  }

  async function destroyWorkspace(e: MouseEvent) {
    e.stopPropagation();
    await invoke(SeelenCommand.DestroyWorkspace, {
      workspaceId: workspace.id,
    });
  }

  async function handleNameChange() {
    const newName = workspaceName.trim();
    if (newName === (workspace.name || "")) return;
    try {
      await invoke(SeelenCommand.RenameWorkspace, {
        workspaceId: workspace.id,
        name: newName || null,
      });
    } catch (error) {
      console.error("Failed to rename workspace:", error);
      workspaceName = workspace.name || "";
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      handleNameChange();
    } else if (e.key === "Escape") {
      workspaceName = workspace.name || "";
    }
  }
</script>

<div
  {@attach droppable.attach}
  class="workspace"
  class:workspace-active={active}
  class:workspace-viewing={viewing}
  class:workspace-drop-target={droppable.isDropTarget}
  role="button"
  tabindex="0"
  onmouseenter={onHover}
  onclick={(e) => {
    e.stopPropagation();
    switchWorkspace();
  }}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      switchWorkspace();
    }
  }}
>
  <div class="workspace-header">
    <input
      type="text"
      bind:value={workspaceName}
      data-skin="transparent"
      class="workspace-name-input"
      placeholder={`Workspace ${index + 1}`}
      onblur={handleNameChange}
      onkeydown={handleKeyDown}
      onclick={(e) => e.stopPropagation()}
    />
    <button data-skin="transparent" onclick={destroyWorkspace}>
      <Icon iconName="TbX" />
    </button>
  </div>

  <div class="workspace-preview">
    <Wallpaper definition={wallpaper} static muted />
  </div>
</div>
