<script lang="ts">
  import { onMount } from "svelte";
  import type { ClassValue } from "svelte/elements";

  interface Props {
    src: string;
    class?: ClassValue;
    [key: string]: any;
  }

  let { src, class: className, ...rest }: Props = $props();

  let svgContent = $state<string | null>(null);
  let error = $state<string | null>(null);

  async function fetchSVG() {
    try {
      const response = await fetch(src);
      if (!response.ok) {
        throw new Error(`Failed to fetch SVG: ${response.statusText}`);
      }
      const svgText = await response.text();
      svgContent = svgText;
    } catch (err: any) {
      error = err?.message;
    }
  }

  onMount(() => {
    fetchSVG();
  });

  $effect(() => {
    svgContent = null;
    error = null;
    fetchSVG();
  });
</script>

{#if !error && svgContent}
  <i {...rest} class={["inline-svg", className]}>
    {@html svgContent}
  </i>
{/if}

<style>
  .inline-svg :global(> svg) {
    width: 100%;
    height: 100%;
  }
</style>
