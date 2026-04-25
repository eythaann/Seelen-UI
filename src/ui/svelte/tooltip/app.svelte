<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { state } from "./state.svelte";

  $effect(() => {
    Widget.getCurrent().ready({ show: true });
  });

  const lines = $derived((state.text ?? "").split("\n"));
</script>

<div class={["slu-std-surface", "tooltip"]} data-showing={state.showing}>
  {#each lines as line}
    <p>{line}</p>
  {/each}
</div>

<style>
  .tooltip {
    width: max-content;
    opacity: 0;

    p {
      margin-bottom: 0.8em;

      &:last-child {
        margin-bottom: 0;
      }
    }
  }

  .tooltip[data-showing="true"] {
    opacity: 1;
  }
</style>
