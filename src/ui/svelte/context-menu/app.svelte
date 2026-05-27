<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { state } from "./state.svelte";
  import MenuItem from "./MenuItem.svelte";
  import Submenu from "./Submenu.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  function onMouseLeave() {
    // Submenus close themselves when the mouse leaves their window
    if (state.isSubmenu) {
      Widget.self.hide();
    }
  }
</script>

<div class="slu-std-popover context-menu" onmouseleave={onMouseLeave}>
  {#each state.data?.items || [] as item}
    {#if item.type === "Separator"}
      <hr />
    {:else if item.type === "Item"}
      <MenuItem {item} />
    {:else if item.type === "Submenu"}
      <Submenu {item} />
    {/if}
  {/each}
</div>
