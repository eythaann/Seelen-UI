<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { globalState } from "./state.svelte";
  import TaskItem from "./components/TaskItem.svelte";

  $effect(() => {
    Widget.self.ready({ show: false });
  });

  const box = $derived.by(() => {
    const monitor = globalState.activeMonitor;
    if (!monitor) {
      return null;
    }

    const { rect, scaleFactor } = monitor;

    return {
      x: rect.left,
      y: rect.top,
      // we reduce the size and later scale it, to get the correct display by dpi aware
      width: (rect.right - rect.left) / scaleFactor,
      height: (rect.bottom - rect.top) / scaleFactor,
      scale: scaleFactor,
    };
  });
</script>

<div
  class="task-switcher-window"
  role="dialog"
  tabindex="-1"
  onclick={() => {
    globalState.showing = false;
  }}
  onkeypress={() => {}}
>
  {#if box}
    <div
      class="task-switcher-monitor"
      style:position="fixed"
      style:left={box.x + "px"}
      style:top={box.y + "px"}
      style:width={box.width + "px"}
      style:height={box.height + "px"}
      style:transform={`scale(${box.scale})`}
      style:transform-origin="left top"
    >
      <div class="task-switcher">
        {#each globalState.windows as window, index (window.hwnd)}
          <TaskItem task={window} {index} />
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    overflow: hidden;
    background: transparent;
  }
</style>
