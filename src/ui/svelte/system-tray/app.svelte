<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { DragDropProvider } from "@dnd-kit/svelte";
  import { onDestroy } from "svelte";
  import { createDragDropManager } from "libs/ui/dnd";
  import { iconsOrder, sortIcons, sortKey, state } from "./state.svelte";
  import TrayItem from "./TrayItem.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  // Workaround for https://github.com/clauderic/dnd-kit/issues/2112
  const manager = createDragDropManager();
  onDestroy(() => manager.destroy());

  const GUIDS_TO_IGNORE = [
    "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // speaker volument icon
    "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // network icon
    "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // battery icon
  ];

  const items = $derived(
    sortIcons(
      state.trayItems.filter(
        (item) => item.is_visible && (!item.guid || !GUIDS_TO_IGNORE.includes(item.guid)),
      ),
      iconsOrder.value,
    ),
  );

  function moveItem(sourceKey: string, targetKey: string) {
    const keys = items.map(sortKey);
    const from = keys.indexOf(sourceKey);
    const to = keys.indexOf(targetKey);
    if (from === -1 || to === -1) {
      return;
    }
    keys.splice(to, 0, keys.splice(from, 1)[0]!);
    // keep the position of icons whose app is not running right now
    const absent = iconsOrder.value.filter((key) => !keys.includes(key));
    iconsOrder.value = [...keys, ...absent];
  }
</script>

<div class={["slu-std-popover", "system-tray"]}>
  <DragDropProvider
    {manager}
    onDragOver={(event) => {
      const { source, target } = event.operation;
      if (source && target && source.id !== target.id) {
        moveItem(source.id as string, target.id as string);
      }
    }}
  >
    {#each items as item, idx (sortKey(item))}
      <TrayItem {item} {idx} />
    {/each}
  </DragDropProvider>
</div>
