<script lang="ts">
  import { state as gState } from "./state.svelte";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { DragDropProvider } from "@dnd-kit/svelte";
  import { move } from "@dnd-kit/helpers";
  import { onDestroy } from "svelte";
  import { createDragDropManager } from "libs/ui/dnd.ts";
  import TrayEntry from "./TrayEntry.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  const GUIDS_TO_IGNORE = [
    "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // speaker volument icon
    "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // network icon
    "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // battery icon
  ];

  const items = $derived(
    gState.trayItems.filter(
      (item) => item.isVisible && (!item.guid || !GUIDS_TO_IGNORE.includes(item.guid)),
    ),
  );

  // Workaround for https://github.com/clauderic/dnd-kit/issues/2112
  const manager = createDragDropManager();
  onDestroy(() => manager.destroy());

  // svelte-ignore state_referenced_locally
  let itemsSnapshop = $state.raw(items);
  $effect(() => {
    manager.actions.stop({ canceled: true });
    itemsSnapshop = items;
  });

  function handleDragOver(event: any) {
    const currentKeys = itemsSnapshop.map((item) => item.registryKey);
    const newKeys = move(currentKeys, event);
    if (newKeys === currentKeys) {
      return;
    }
    const byKey = new Map(itemsSnapshop.map((item) => [item.registryKey, item]));
    itemsSnapshop = newKeys.map((key) => byKey.get(key)!);
  }

  function handleDragEnd(event: any) {
    if (event.canceled) {
      itemsSnapshop = items;
      return;
    }
    const currentKeys = items.map((item) => item.registryKey);
    const newKeys = itemsSnapshop.map((item) => item.registryKey);
    if (newKeys.every((key, i) => key === currentKeys[i])) {
      return;
    }
    invoke(SeelenCommand.SetSystemTrayIconsOrder, { keys: newKeys });
  }
</script>

<div class={["slu-std-popover", "system-tray"]}>
  <DragDropProvider {manager} onDragOver={handleDragOver} onDragEnd={handleDragEnd}>
    {#each itemsSnapshop as item, index (item.registryKey)}
      <TrayEntry {item} {index} />
    {/each}
  </DragDropProvider>
</div>
