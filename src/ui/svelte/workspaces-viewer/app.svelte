<script lang="ts">
  import Monitor from "./app/Monitor.svelte";
  import { state } from "./state.svelte";
  import { Widget } from "@seelen-ui/lib";

  $effect(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class="workspaces-viewer">
  {#each state.monitors as monitor}
    <Monitor {monitor} />
  {/each}
</div>
