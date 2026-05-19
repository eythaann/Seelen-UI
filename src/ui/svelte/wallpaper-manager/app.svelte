<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import MonitorContainers from "./modules/Monitor/infra.svelte";

  $effect(() => {
    Widget.self.ready();
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
