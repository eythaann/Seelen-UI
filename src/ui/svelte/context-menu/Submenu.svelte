<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import type { ContextMenuItem } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "./state.svelte";

  interface SubmenuProps {
    item: Extract<ContextMenuItem, { type: "Submenu" }>;
  }

  let { item }: SubmenuProps = $props();

  function handleClick() {
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: {
        identifier: item.identifier,
        items: item.items,
      },
      forwardTo: state.forwardTo || state.owner,
    });
  }
</script>

<button class="menu-item" data-skin="transparent" onclick={handleClick}>
  <Icon iconName={item.icon as any} />
  <span class="menu-item-label">{item.label}</span>
  <Icon class="menu-item-chevron" iconName="FaChevronRight" />
</button>
