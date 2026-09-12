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
  let mounted = $state(false);

  $effect(() => {
    const timeout = setTimeout(() => {
      mounted = true;
    }, 350);
    return () => clearTimeout(timeout);
  });

  function dockItemTransition(
    node: HTMLElement,
    { horizontal = true }: { horizontal?: boolean } = {},
  ): TransitionConfig {
    if (!mounted) {
      return { duration: 0 };
    }

    const style = getComputedStyle(node);
    const targetOpacity = +style.opacity || 1;
    const targetDimension = horizontal ? node.offsetWidth : node.offsetHeight;
    const fallbackSize =
      parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-item-size")) ||
      48;
    const dimension = targetDimension > 0 ? targetDimension : fallbackSize;

    return {
      duration: 220,
      easing: cubicOut,
      css: (t: number) => {
        const scale = 0.6 + 0.4 * t;
        const currentSize = dimension * t;
        const sizeStyle = horizontal
          ? `max-width: ${currentSize}px; width: ${currentSize}px;`
          : `max-height: ${currentSize}px; height: ${currentSize}px;`;

        return `
          opacity: ${t * targetOpacity};
          transform: scale(${scale});
          ${sizeStyle}
          overflow: hidden;
          flex-shrink: 0;
        `;
      },
    };
  }
</script>

<div class="weg-items-{id}" data-empty={items.length === 0}>
  {#each items as item (item.id)}
    <div
      class="weg-item-animator"
      animate:flip={{ duration: dockIsDragging.value ? 0 : 220, easing: cubicOut }}
      transition:dockItemTransition={{ horizontal: isHorizontal }}
    >
      <DraggableItem {item} index={itemIndexById.get(item.id) ?? 0}>
        <WegItemSwitch {item} />
      </DraggableItem>
    </div>
  {/each}
</div>
