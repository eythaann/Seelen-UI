<script lang="ts">
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
</script>

<div class="pinned-view">
  <div class="pinned-view-list">
    {#each globalState.pinnedItems as item (globalState.getItemId(item))}
      <AppItem {item} pinned={true} class="pinned-view-item" />
    {/each}
  </div>

  {#if globalState.pinnedItems.length === 0}
    <div class="pinned-view-empty">
      {$t("no_pinned_apps")}
    </div>
  {/if}
</div>
