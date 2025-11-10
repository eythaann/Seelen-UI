<script lang="ts">
  import Monitor from "./app/Monitor.svelte";
  import { state } from "./state.svelte";

</script>

<div class="workspaces-viewer">
  {#each state.monitors as monitor}
    <Monitor {monitor} />
  {/each}
</div>
