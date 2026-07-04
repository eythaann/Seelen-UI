<script lang="ts">
  import { setContext } from "svelte";
  import { requestPositioningOfLeaves } from "../application.ts";
  import { state as wmState } from "../../state.svelte.ts";
  import Container from "./Container.svelte";
  import { NodeUtils } from "../../utils.ts";
  import { TREE_CONTEXT_KEY } from "../domain.ts";
  import type { TwmRuntimeTree } from "@seelen-ui/lib/types";

  interface Props {
    monitorId: string;
  }

  let { monitorId }: Props = $props();

  // `wmState.getLayout` returns a fresh object reference on every WMTreeChanged event,
  // even when this monitor's workspace tree didn't actually change (the backend
  // replaces the whole render tree, not just the affected workspace).
  let lastValidLayout: TwmRuntimeTree | null = null;

  const layout = $derived.by(() => {
    const next = wmState.getLayout(monitorId);
    if (JSON.stringify(next) !== JSON.stringify(lastValidLayout)) {
      lastValidLayout = next;
    }
    return lastValidLayout;
  });

  setContext(TREE_CONTEXT_KEY, {
    get tree() {
      return layout;
    },
  });

  let overlayVisible = $derived.by(() => {
    if (!layout || wmState.paused) {
      return false;
    }

    if (["Progman", "SysListView32"].includes(wmState.focusedApp.class)) {
      return true;
    }

    if (!NodeUtils.contains(layout, layout.root, wmState.focusedApp.hwnd)) {
      return false;
    }

    return true;
  });

  $effect(() => {
    wmState.forceRepositioning; // subscription
    if (layout) {
      requestPositioningOfLeaves(wmState);
    }
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
