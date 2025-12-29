<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "./i18n";
  import { Widget } from "@seelen-ui/lib";
  import PinnedView from "./components/PinnedView.svelte";
  import AllAppsView from "./components/AllAppsView.svelte";
  import { globalState } from "./state.svelte";
  import { StartDisplayMode, StartView } from "./constants";
  import { Icon } from "libs/ui/svelte/components/Icon";

  onMount(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class="apps-menu" class:fullscreen={globalState.displayMode === StartDisplayMode.Fullscreen}>
  <div class="apps-menu-header">
    <input type="search" data-skin="transparent" placeholder="Applications" />

    <button
      data-skin="default"
      onclick={() => {
        globalState.view =
          globalState.view === StartView.Favorites ? StartView.All : StartView.Favorites;
      }}
    >
      {#if globalState.view === StartView.Favorites}
        {$t("all")}
      {:else}
        {$t("back")}
      {/if}
    </button>
  </div>

  <div class="apps-menu-body">
    {#if globalState.view === StartView.Favorites}
      <PinnedView />
    {:else if globalState.view === StartView.All}
      <AllAppsView />
    {/if}
  </div>

  <div class="apps-menu-footer">
    <button
      data-skin="transparent"
      onclick={() => {
        globalState.displayMode =
          globalState.displayMode === StartDisplayMode.Normal
            ? StartDisplayMode.Fullscreen
            : StartDisplayMode.Normal;
      }}
    >
      <Icon iconName="BsWindows" />
    </button>
  </div>
</div>
