<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import { iconPackManager, type IconState } from "./common.svelte.ts";
  import { prefersDarkColorScheme } from "../../runes/DarkMode.svelte.ts";
  import MissingIcon from "./MissingIcon.svelte";

  interface Props {
    name: string;
    class?: ClassValue;
    [key: string]: any;
  }

  let { name, class: className, ...imgProps }: Props = $props();

  let state: IconState = $derived.by(() => {
    // Depend on _version to trigger reactivity when icon pack changes
    iconPackManager._version;
    const icon = iconPackManager.value.getMissingIcon();
    if (icon) {
      return {
        src: (prefersDarkColorScheme.value ? icon.dark : icon.light) || icon.base,
        mask: icon.mask,
        isAproximatelySquare: icon.isAproximatelySquare,
      };
    }
    return { src: null, mask: null, isAproximatelySquare: false };
  });
</script>

{#if state.src}
  <figure
    {...imgProps}
    class={["slu-icon-outer", className]}
    data-shape={state.isAproximatelySquare ? "square" : "unknown"}
  >
    <img src={state.src} alt="" />
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
