<script lang="ts">
  import { requestPositioningOfLeaves } from "../application.ts";
  import { state } from "../../shared/state.svelte.ts";
  import Container from "./Container.svelte";

  interface Props {
    monitorId: string;
  }

  let { monitorId }: Props = $props();

  let layout = $derived(state.getLayout(monitorId));

  // Retrigger repositioning when dependencies change
  $effect(() => {
    layout;
    state.forceRepositioning;
    requestPositioningOfLeaves(state);
  });

  // Update body opacity based on overlay visibility
  $effect(() => {
    document.body.style.opacity = state.overlayVisible ? "1" : "0";
  });
</script>

{#if layout}
  <Container node={layout} />
{/if}

<style>
  @import "./index.css";
</style>
