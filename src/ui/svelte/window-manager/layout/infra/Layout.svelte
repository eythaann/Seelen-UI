<script lang="ts">
  import { setContext } from "svelte";
  import { requestPositioningOfLeaves } from "../application.ts";
  import { state as wmState } from "../../state.svelte.ts";
  import Container from "./Container.svelte";
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

  // z-order is handled by the backend (raised on foreground of managed windows),
  // so the overlay doesn't need to be hidden when a non managed window is focused.
  let overlayVisible = $derived(!!layout && !wmState.paused);

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
