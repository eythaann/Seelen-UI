<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon, MissingIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { delayedFocused, previews } from "../../state/windows.svelte.ts";

  interface Props {
    title: string;
    hwnd: number;
  }

  let { title, hwnd }: Props = $props();

  const preview = $derived(previews.value[hwnd]);

  function onClick() {
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd,
      wasFocused: delayedFocused.value?.hwnd === hwnd,
    });
  }

  function onAuxClick(e: MouseEvent) {
    if (e.button === 1) {
      invoke(SeelenCommand.WegCloseApp, { hwnd });
    }
  }

  function onClose(e: MouseEvent) {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  }
</script>

<div
  role="button"
  tabindex="0"
  class="weg-item-preview"
  onclick={onClick}
  onauxclick={onAuxClick}
  onkeypress={() => {}}
>
  <div class="weg-item-preview-topbar">
    <div class="weg-item-preview-title">{title}</div>
    <button data-skin="transparent" onclick={onClose}>
      <Icon iconName="IoClose" />
    </button>
  </div>
  <div class="weg-item-preview-image-container">
    {#if preview}
      <img class="weg-item-preview-image" src="data:image/webp;base64,{preview.data}" alt={title} />
    {:else}
      <MissingIcon class="weg-item-no-preview" />
    {/if}
  </div>
</div>
