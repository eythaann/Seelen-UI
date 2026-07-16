<script lang="ts">
  import type { Snippet } from "svelte";
  import type { SwItem } from "../types.ts";
  import { settingsState, isHorizontalDock } from "../state/settings.svelte.ts";
  import { t } from "../i18n/index.ts";
  import { interactables, getWindowsForItem } from "../state/windows.svelte.ts";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { RestrictToHorizontalAxis, RestrictToVerticalAxis } from "@dnd-kit/abstract/modifiers";
  import { dockState } from "../state/items.svelte.ts";

  interface Props {
    item: SwItem;
    index: number;
    children: Snippet;
  }

  let { item, index, children }: Props = $props();

  const sortable = createSortable({
    get id() {
      return item.id;
    },
    get index() {
      return index;
    },
    get disabled() {
      return dockState.isReorderDisabled;
    },
    get modifiers() {
      return [isHorizontalDock() ? RestrictToHorizontalAxis : RestrictToVerticalAxis];
    },
  });

  const tooltip = $derived.by(() => {
    switch (item.type) {
      case "AppOrFile": {
        const windows = getWindowsForItem(item as any, interactables.value);
        if (windows.length === 0) return (item as any).displayName;
        return undefined;
      }
      case "Media":
        return $t("media.label");
      case "StartMenu":
        return $t("start.label");
      case "ShowDesktop":
        return $t("show_desktop.label");
      case "TrashBin":
        return $t("trash_bin.label");
      default:
        return undefined;
    }
  });
</script>

<div
  {@attach sortable.attach}
  style="opacity: {sortable.isDragging ? 0.3 : 1}"
  data-dragging={sortable.isDragging}
  data-item-id={item.id}
  class="weg-item-drag-container"
  class:dragging={sortable.isDragging}
  data-tooltip={tooltip}
  data-tooltip-align-x={settingsState.popupAlignX}
  data-tooltip-align-y={settingsState.popupAlignY}
>
  {@render children()}
</div>
