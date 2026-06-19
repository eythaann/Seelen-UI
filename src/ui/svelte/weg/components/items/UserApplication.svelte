<script lang="ts">
  import type { AppOrFileWegItem } from "../../types.ts";
  import { settingsState } from "../../state/settings.svelte.ts";
  import { interactables, getWindowsForItem } from "../../state/windows.svelte.ts";
  import UserApplicationItem from "./UserApplicationItem.svelte";

  interface Props {
    item: AppOrFileWegItem;
    isOverlay?: boolean;
  }

  let { item, isOverlay = false }: Props = $props();

  const windows = $derived(getWindowsForItem(item, interactables.value));
  const settings = $derived(settingsState.value as any);
  const showAsSeparatedItems = $derived(settings?.splitWindows);
</script>

{#if showAsSeparatedItems && windows.length > 1}
  <div
    class="weg-split-items"
    style="display: flex; align-items: center; gap: {settings?.spaceBetweenItems ?? 0}px;"
  >
    {#each windows as win (win.hwnd)}
      <UserApplicationItem {item} windows={[win]} {isOverlay} />
    {/each}
  </div>
{:else}
  <UserApplicationItem {item} {windows} {isOverlay} />
{/if}
