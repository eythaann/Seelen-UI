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

  let previousSrc = $state<string | null>(null);
  let icon: IconState = $derived.by(() => {
    // Depend on _version to trigger reactivity when icon pack changes
    iconPackManager._version;
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
    if (!mounted.value) {
      IconPackManager.requestIconExtraction({ path, umid });
      mounted.value = true;
    }

    // Trigger icon extraction when src goes from non-null to null
    if (previousSrc !== null && icon.src === null) {
      IconPackManager.requestIconExtraction({ path, umid });
    }
    previousSrc = icon.src;
  });
</script>

{#if icon.src}
  <figure
    {...imgProps}
    class={["slu-icon-outer", className]}
    data-shape={icon.isAproximatelySquare ? "square" : "unknown"}
  >
    <img src={icon.src} alt="" loading={lazy ? "lazy" : "eager"} />
    {#if icon.mask}
      <div class="slu-icon-mask" style="mask-image: url('{icon.mask}')"></div>
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
