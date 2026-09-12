<script lang="ts">
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import type { TransitionConfig } from "svelte/transition";
  import type { SwItem } from "../types.ts";
  import { dockIsDragging } from "../state/hidden.svelte.ts";
  import { isHorizontalDock } from "../state/settings.svelte.ts";
  import DraggableItem from "./DraggableItem.svelte";
  import WegItemSwitch from "./WegItemSwitch.svelte";

  interface Props {
    id: string;
    items: SwItem[];
    itemIndexById: Map<string, number>;
  }

  let { id, items, itemIndexById }: Props = $props();

  const isHorizontal = $derived(isHorizontalDock());

  let isInitialLoad = true;
  if (typeof window !== "undefined") {
    setTimeout(() => {
      isInitialLoad = false;
    }, 600);
  }

  function dockItemExit(
    node: HTMLElement,
    { horizontal = true }: { horizontal?: boolean } = {},
  ): TransitionConfig {
    console.info("[WEG] dockItemExit starting for node");
    const style = getComputedStyle(node);
    const targetOpacity = +style.opacity || 1;
    const targetDimension = horizontal ? node.offsetWidth : node.offsetHeight;
    const fallbackSize =
      parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-item-size")) ||
      48;
    const dimension = targetDimension > 0 ? targetDimension : fallbackSize;
    const gap =
      parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-space-between-items")) ||
      8;

    return {
      duration: 220,
      easing: cubicOut,
      css: (t: number) => {
        const scale = 0.6 + 0.4 * t;
        const currentSize = dimension * t;
        const currentMargin = (1 - t) * -gap;
        const sizeStyle = horizontal
          ? `max-width: ${currentSize}px; width: ${currentSize}px; min-width: 0px; margin-right: ${currentMargin}px;`
          : `max-height: ${currentSize}px; height: ${currentSize}px; min-height: 0px; margin-bottom: ${currentMargin}px;`;

        return `opacity: ${t * targetOpacity}; transform: scale(${scale}); ${sizeStyle} overflow: hidden; flex-shrink: 0;`;
      },
    };
  }

  function dockItemEnter(
    node: HTMLElement,
    { horizontal = true }: { horizontal?: boolean } = {},
  ): TransitionConfig {
    if (isInitialLoad) {
      return { duration: 0 };
    }

    console.info("[WEG] dockItemEnter starting for node");
    const targetDimension = horizontal ? node.offsetWidth : node.offsetHeight;
    const fallbackSize =
      parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-item-size")) ||
      48;
    const dimension = targetDimension > 0 ? targetDimension : fallbackSize;
    const gap =
      parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-space-between-items")) ||
      8;

    return {
      duration: 220,
      easing: cubicOut,
      css: (t: number) => {
        const scale = 0.6 + 0.4 * t;
        const currentSize = dimension * t;
        const currentMargin = (1 - t) * -gap;
        const sizeStyle = horizontal
          ? `max-width: ${currentSize}px; width: ${currentSize}px; min-width: 0px; margin-right: ${currentMargin}px;`
          : `max-height: ${currentSize}px; height: ${currentSize}px; min-height: 0px; margin-bottom: ${currentMargin}px;`;

        return `opacity: ${t}; transform: scale(${scale}); ${sizeStyle} overflow: hidden; flex-shrink: 0;`;
      },
    };
  }
</script>

<div class="weg-items-{id}" data-empty={items.length === 0}>
  {#each items as item (item.id)}
    <div
      class="weg-item-animator"
      in:dockItemEnter|global={{ horizontal: isHorizontal }}
      out:dockItemExit|global={{ horizontal: isHorizontal }}
    >
      <DraggableItem {item} index={itemIndexById.get(item.id) ?? 0}>
        <WegItemSwitch {item} />
      </DraggableItem>
    </div>
  {/each}
</div>
