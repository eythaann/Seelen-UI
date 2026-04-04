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

<div class="slu-std-popover media-popup">
  {#if globalState.view === "main"}
    <MainView />
  {:else}
    <DeviceView onBack={handleBack} />
  {/if}
</div>
