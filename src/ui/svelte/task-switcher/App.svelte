<script lang="ts">
  import { onMount } from "svelte";
  import { Widget } from "@seelen-ui/lib";
  import { globalState } from "./state.svelte";
  import TaskItem from "./components/TaskItem.svelte";

  const widget = Widget.getCurrent();

  onMount(() => {
    widget.webview.onResized(() => {
      widget.webview.center();
    });
    widget.ready();
  });
</script>

<div class="task-switcher">
  {#each globalState.windows as window, index (window.hwnd)}
    <TaskItem data={window} {index} />
  {/each}
</div>

<style>
  :global(body) {
    overflow: hidden;
    background: transparent;
  }

  :global(#root) {
    width: min-content;
    height: min-content;
  }
</style>
