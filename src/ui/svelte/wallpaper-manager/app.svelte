<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { gState } from "./state.svelte.ts";
  import MonitorContainers from "./modules/Monitor/infra.svelte";

  let isReady = $state(false);

  $effect(() => {
    Widget.self.ready().then(() => {
      isReady = true;
    });
  });

  // Re-apply positioning when monitors change (only after window is visible).
  $effect(() => {
    gState.monitors;
    if (isReady) {
      invoke(SeelenCommand.SetAsWallpaper).catch(console.error);
    }
  });
</script>

<MonitorContainers />

<style>
  :global(body) {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: #000;
  }

  :global(#root) {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
