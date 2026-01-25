<script lang="ts">
  import { globalState } from "./state.svelte";
  import { Widget } from "@seelen-ui/lib";
  import MainView from "./components/MainView.svelte";
  import DeviceView from "./components/DeviceView.svelte";

  function handleBack() {
    globalState.view = "main";
    globalState.selectedDeviceId = null;
  }

  $effect(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class="slu-standard-popover media-popup">
  {#if globalState.view === "main"}
    <MainView />
  {:else}
    <DeviceView onBack={handleBack} />
  {/if}
</div>

<style>
  .media-popup {
    background: var(--config-background-color, var(--color-gray-100));
    color: var(--config-foreground-color, var(--color-gray-900));
    border-radius: 8px;
    overflow: hidden;
  }
</style>
