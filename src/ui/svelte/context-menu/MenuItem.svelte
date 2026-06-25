<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import type { ContextMenuItem, ContextMenuCallbackPayload } from "@seelen-ui/lib/types";
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

    if (!target) {
      console.warn("Context menu has no target to emit to");
      return;
    }

    if (item.disabled) {
      return;
    }

    if (item.checked !== null) {
      // Toggle optimistic state
      internalChecked = !internalChecked;
    }

    emitTo<ContextMenuCallbackPayload>(target, item.callbackEvent, {
      key: item.key,
      value: item.value,
      checked: item.checked !== null ? internalChecked : null,
      meta: gState.data?.meta,
    });

    if (item.checked === null) {
      Widget.self.hide();
    }
  }
</script>

<button class="menu-item" class:danger={item.danger} disabled={item.disabled} data-skin="transparent" onclick={handleClick}>
  {#if item.checked !== null}
    <input type="checkbox" data-skin="default" checked={internalChecked} />
  {/if}
  {#if item.icon?.startsWith("http://asset.localhost")}
    <img class="menu-item-img-icon" src={item.icon} alt="" />
  {:else}
    <Icon iconName={item.icon as any} />
  {/if}
  <span class="menu-item-label">{item.label}</span>
</button>
