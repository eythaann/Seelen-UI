<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { state } from "./state.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class={["slu-std-surface", "tooltip"]} data-showing={state.showing}>{state.text ?? ""}</div>

<style>
  .tooltip {
    width: max-content;
    opacity: 0;
    white-space: pre-line;
  }

  .tooltip[data-showing="true"] {
    opacity: 1;
  }
</style>
