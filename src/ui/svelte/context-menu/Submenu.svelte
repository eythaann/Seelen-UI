<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { type ContextMenuItem, Alignment } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state, closeOpenSubmenu, setOpenSubmenu } from "./state.svelte";

  interface SubmenuProps {
    item: Extract<ContextMenuItem, { type: "Submenu" }>;
  }

  let { item }: SubmenuProps = $props();

  async function handleMouseEnter(e: MouseEvent) {
    // Capture button ref BEFORE any await — currentTarget becomes null after await
    const button = e.currentTarget as HTMLElement;

    // Close any previously open submenu first
    await closeOpenSubmenu();
    // Track this submenu as the open one
    setOpenSubmenu(item.identifier);

    const rect = button.getBoundingClientRect();
    const dpr = window.devicePixelRatio;

    // Convert button position from CSS (logical) px to physical screen coords.
    // window.screenX/Y gives the window's top-left in CSS px.
    const buttonRight = (window.screenX + rect.right) * dpr;
    const buttonTop = (window.screenY + rect.top) * dpr;
    const buttonBottom = (window.screenY + rect.bottom) * dpr;

    // Rough submenu height estimate: items × 36px row height + 16px padding
    const estimatedHeight = (item.items.length * 36 + 16) * dpr;
    const screenBottom = window.screen.height * dpr;

    // Smart vertical alignment: open downward if space available, otherwise upward
    let alignY: Alignment | null;
    let desiredY: number;
    if (buttonTop + estimatedHeight <= screenBottom) {
      alignY = null; // Start — submenu top aligns with button top
      desiredY = buttonTop;
    } else {
      alignY = Alignment.End; // End — submenu bottom aligns with button bottom
      desiredY = buttonBottom;
    }

    invoke(SeelenCommand.TriggerContextMenu, {
      menu: {
        identifier: item.identifier,
        items: item.items,
        alignX: Alignment.Start,
        alignY,
        desiredPosition: {
          x: Math.round(buttonRight),
          y: Math.round(desiredY),
        },
      },
      forwardTo: state.forwardTo || state.owner,
    });
  }
</script>

<button
  class="menu-item"
  data-skin="transparent"
  onclick={handleMouseEnter}
  onmouseenter={handleMouseEnter}
>
  <Icon iconName={item.icon as any} />
  <span class="menu-item-label">{item.label}</span>
  <Icon class="menu-item-chevron" iconName="FaChevronRight" />
</button>
