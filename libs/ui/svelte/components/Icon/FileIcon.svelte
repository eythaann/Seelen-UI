<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import type { SeelenCommandGetIconArgs } from "@seelen-ui/lib/types";
  import { IconPackManager } from "@seelen-ui/lib";
  import { iconPackManager, type IconState } from "./common.svelte.ts";
  import MissingIcon from "./MissingIcon.svelte";
  import { prefersDarkColorScheme } from "../../runes/DarkMode.svelte.ts";

  interface Props extends SeelenCommandGetIconArgs {
    class?: ClassValue;
    lazy?: boolean;
    [key: string]: any;
  }

  let { path, umid, class: className, lazy, ...imgProps }: Props = $props();

  let mounted = { value: false };
  let state: IconState = $derived.by(() => {
    const icon = iconPackManager.value.getIcon({ path, umid });
    if (icon) {
      return {
        src: (prefersDarkColorScheme.value ? icon.dark : icon.light) || icon.base,
        mask: icon.mask,
        isAproximatelySquare: icon.isAproximatelySquare,
      };
    }

    return { src: null, mask: null, isAproximatelySquare: false };
  });

  // Watch for src becoming null (trigger icon extraction)
  $effect(() => {
    if (state.src === null || !mounted.value) {
      IconPackManager.requestIconExtraction({ path, umid });
      mounted.value = true;
    }
  });
</script>

{#if state.src}
  <figure
    {...imgProps}
    class={["slu-icon-outer", className]}
    data-shape={state.isAproximatelySquare ? "square" : "unknown"}
  >
    <img src={state.src} alt="" loading={lazy ? "lazy" : "eager"} />
    {#if state.mask}
      <div class="slu-icon-mask" style="mask-image: url('{state.mask}')"></div>
    {/if}
  </figure>
{:else}
  <MissingIcon {...imgProps} class={className} />
{/if}

<style>
  :global(.slu-icon-outer) {
    position: relative;
  }

  :global(.slu-icon-outer img) {
    height: 100%;
    object-fit: contain;
  }

  :global(.slu-icon-mask) {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    mask-repeat: no-repeat;
    mask-size: contain;
    mask-position: center;
    mask-mode: luminance;
    background-color: var(--system-accent-light-color);
  }
</style>
