<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { Icon, MissingIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { previewState } from "./state.svelte.ts";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  $effect(() => {
    if (previewState.currentInteractables.length === 0) {
      Widget.self.hide();
    }
  });

  function onClickPreview(hwnd: number) {
    invoke(SeelenCommand.WegToggleWindowState, { hwnd, wasFocused: false });
  }

  function onClosePreview(e: MouseEvent, hwnd: number) {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  function onAuxClickPreview(e: MouseEvent, hwnd: number) {
    if (e.button === 1) {
      invoke(SeelenCommand.WegCloseApp, { hwnd });
    }
  }
</script>

<div class="weg-item-preview-container slu-std-popover">
  <div class="weg-item-preview-list">
    {#each previewState.currentInteractables as win (win.hwnd)}
      {@const preview = previewState.previews.value[win.hwnd]}
      <div
        role="button"
        tabindex="0"
        class="weg-item-preview"
        onclick={() => onClickPreview(win.hwnd)}
        onauxclick={(e) => onAuxClickPreview(e, win.hwnd)}
        onkeypress={() => {}}
      >
        <div class="weg-item-preview-topbar">
          <div class="weg-item-preview-title">{win.title}</div>
          <button data-skin="transparent" onclick={(e) => onClosePreview(e, win.hwnd)}>
            <Icon iconName="IoClose" />
          </button>
        </div>
        <div class="weg-item-preview-image-container">
          {#if preview}
            <img
              class="weg-item-preview-image"
              src="data:image/webp;base64,{preview.data}"
              alt={win.title}
            />
          {:else}
            <MissingIcon class="weg-item-no-preview" />
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>
