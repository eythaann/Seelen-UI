<script lang="ts">
  import { setContext } from "svelte";
  import { requestPositioningOfLeaves } from "../application.ts";
  import { state } from "../../state.svelte.ts";
  import Container from "./Container.svelte";
  import { NodeUtils } from "../../utils.ts";
  import { TREE_CONTEXT_KEY } from "../domain.ts";

  interface Props {
    monitorId: string;
  }

  let { monitorId }: Props = $props();

  let layout = $derived(state.getLayout(monitorId));

  setContext(TREE_CONTEXT_KEY, {
    get tree() {
      return layout;
    },
  });

  let overlayVisible = $derived.by(() => {
    if (!layout || state.pausedByMonitor[monitorId] || state.paused) {
      return false;
    }

    if (["Progman", "SysListView32"].includes(state.focusedApp.class)) {
      return true;
    }

    if (!NodeUtils.contains(layout, layout.root, state.focusedApp.hwnd)) {
      return false;
    }

    return true;
  });

  /* $effect(() => {
    const observer = new ResizeObserver(() => {
      requestPositioningOfLeaves(state);
    });
    observer.observe(document.body, {
      box: "border-box",
    });

    return () => {
      observer.disconnect();
    };
  }); */

  // Retrigger repositioning when dependencies change
  $effect(() => {
    layout;
    state.forceRepositioning;
    requestPositioningOfLeaves(state);
  });

  // Update body opacity based on overlay visibility
  $effect(() => {
    document.body.style.opacity = overlayVisible ? "1" : "0";
  });
</script>

{#if layout}
  <Container nodeId={layout.root} {overlayVisible} />
{/if}

<style>
  @import "./index.css";
</style>
