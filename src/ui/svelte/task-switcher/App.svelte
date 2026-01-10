<script lang="ts">
  import { onMount } from "svelte";
  import { Widget } from "@seelen-ui/lib";
  import { globalState } from "./state.svelte";
  import TaskItem from "./components/TaskItem.svelte";

  const widget = Widget.getCurrent();

  onMount(() => {
    widget.ready();
  });
</script>

<div
  class="task-switcher-overlay"
  role="dialog"
  tabindex="-1"
  onclick={() => {
    globalState.showing = false;
  }}
  onkeypress={() => {}}
>
  <div class="task-switcher">
    {#each globalState.windows as window, index (window.hwnd)}
      <TaskItem task={window} {index} />
    {/each}
  </div>
</div>

<style>
  :global(body) {
    overflow: hidden;
    background: transparent;
  }
</style>
