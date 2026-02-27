<script lang="ts">
  import { onMount } from "svelte";
  import { Widget } from "@seelen-ui/lib";
  import MonitorContainers from "./modules/Monitor/infra.svelte";

  onMount(() => {
    // Manually show the widget to avoid the focus call on ready,
    // as this unfocusable widget causes the wallpaper to not be correctly
    // positioned under the desktop.
    Widget.self.ready({ show: false }).then(() => {
      Widget.self.show();
    });
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
