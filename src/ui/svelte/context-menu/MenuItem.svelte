<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import type { ContextMenuItem } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { emitTo } from "@tauri-apps/api/event";
  import { state as gState } from "./state.svelte";

  interface MenuItemProps {
    item: Extract<ContextMenuItem, { type: "Item" }>;
  }

  let { item }: MenuItemProps = $props();

  // Optimistic state for checked
  let internalChecked = $state(false);

  $effect.pre(() => {
    internalChecked = item.checked!!;
  });

  function handleClick() {
    let target = gState.forwardTo || gState.owner;
    if (item.disabled || !target) {
      return;
    }

    if (item.checked !== null) {
      // Toggle optimistic state
      internalChecked = !internalChecked;
      // Emit with the new checked value
      emitTo(target, item.callbackEvent, { key: item.key, checked: internalChecked });
    } else {
      emitTo(target, item.callbackEvent, { key: item.key });
      Widget.self.hide(true);
    }
  }
</script>

<button class="menu-item" disabled={item.disabled} data-skin="transparent" onclick={handleClick}>
  {#if item.checked !== null}
    <input type="checkbox" checked={internalChecked} />
  {/if}
  <Icon iconName={item.icon as any} />
  <span class="menu-item-label">{item.label}</span>
</button>
