<script lang="ts">
  import { type StartMenuItem } from "@seelen-ui/lib/types";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import { searchState, getItemKey } from "../state/search.svelte";

  interface Props {
    onContextMenu: (event: MouseEvent, item: StartMenuItem) => void;
  }

  let { onContextMenu }: Props = $props();
</script>

<div class="all-apps-view">
  <div class="all-apps-view-list">
    {#each searchState.searchedItems as item, idx (getItemKey(item))}
      <AppItem {item} {idx} {onContextMenu} lazy />
    {/each}
  </div>

  {#if searchState.searchedItems.length === 0}
    <div class="all-apps-view-empty">
      {searchState.searchFilter === "web" ? $t("web_search") : $t("no_matching_items")}
    </div>
  {/if}
</div>
