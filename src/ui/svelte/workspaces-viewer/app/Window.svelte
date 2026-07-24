<script lang="ts">
  import type { UserAppWindow } from "@seelen-ui/lib/types";
  import { state } from "../state.svelte";
  import { Icon, MissingIcon, FileIcon } from "libs/ui/svelte/components/Icon";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { createDraggable } from "@dnd-kit/svelte";

  interface Props {
    hwnd: number;
    overlay?: boolean;
  }

  let { hwnd, overlay }: Props = $props();

  const windowData = $derived(state.windows.find((w: UserAppWindow) => w.hwnd === hwnd));
  const preview = $derived(state.previews[hwnd]);

  const aspectRatio = $derived(preview ? preview.width / preview.height : 16 / 9);

  // svelte-ignore state_referenced_locally
  const draggable = overlay
    ? null
    : createDraggable({
        get id() {
          return hwnd;
        },
      });
</script>

<div
  {@attach draggable?.attach}
  class="window"
  class:is-dragging={!!draggable?.isDragging}
  class:is-dropping={!!draggable?.isDropping}
  class:is-overlay={!!overlay}
  role="button"
  tabindex="0"
  onclick={(e) => {
    e.stopPropagation();
    invoke(SeelenCommand.WegToggleWindowState, { hwnd, wasFocused: false });
    Widget.self.hide();
  }}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.currentTarget?.click();
    }
  }}
>
  <div class="window-header">
    <FileIcon
      umid={windowData?.umid}
      path={windowData?.relaunch?.icon || windowData?.process?.path}
    />
    <div class="window-title">
      {windowData?.title || hwnd.toString(16)}
    </div>
    <button
      data-skin="transparent"
      onclick={(e) => {
        e.stopPropagation();
        invoke(SeelenCommand.WegCloseApp, { hwnd });
      }}
    >
      <Icon iconName="TbX" />
    </button>
  </div>

  <div class="window-preview-container" style="aspect-ratio: {aspectRatio}">
    {#if preview}
      <img class="window-preview" src={`data:image/webp;base64,${preview.data}`} alt="" />
    {:else}
      <MissingIcon class="window-no-preview" />
    {/if}
  </div>
</div>
