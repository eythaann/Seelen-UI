<script lang="ts">
  import { requestPositioningOfLeaves } from "../application.ts";
  import { state } from "../../shared/state.svelte.ts";
  import Container from "./Container.svelte";

  // Retrigger repositioning when dependencies change
  $effect(() => {
    state.layout;
    state.forceRepositioning;
    requestPositioningOfLeaves(state);
  });

  // Update body opacity based on overlay visibility
  $effect(() => {
    document.body.style.opacity = state.overlayVisible ? "1" : "0";
  });
</script>

{#if state.layout}
  <Container node={state.layout} />
{/if}

<style>
  @import "./index.css";
</style>
