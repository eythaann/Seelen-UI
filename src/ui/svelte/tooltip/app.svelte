<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { state } from "./state.svelte";

  $effect(() => {
    Widget.getCurrent().ready({ show: true });
  });
</script>

<div class={["slu-std-surface", "tooltip"]} data-showing={state.showing}>
  <span>{state.text ?? ""}</span>
</div>

<style>
  .tooltip {
    width: max-content;
    opacity: 0;
  }

  .tooltip[data-showing="true"] {
    opacity: 1;
  }
</style>
