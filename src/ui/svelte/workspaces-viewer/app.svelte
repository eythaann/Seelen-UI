<script lang="ts">
  import Monitor from "./app/Monitor.svelte";
  import { state } from "./state.svelte";
  import { Widget } from "@seelen-ui/lib";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  function onCancel() {
    Widget.getCurrent().webview.hide();
  }
</script>

<div
  class="workspaces-viewer"
  role="menu"
  tabindex="-1"
  onclick={onCancel}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      onCancel();
    }
  }}
>
  {#each state.monitors as monitor}
    <Monitor {monitor} />
  {/each}
</div>

<style>
  :global(body) {
    overflow: hidden;
    background: transparent;
  }
</style>