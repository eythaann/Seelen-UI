<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { ClassValue } from "svelte/elements";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { iconPackManager } from "./common.ts";

  interface Props {
    name: string;
    class?: ClassValue;
    [key: string]: any;
  }

  let { name, class: className, ...imgProps }: Props = $props();

  interface IconState {
    src: string | null;
    mask: string | null;
    isAproximatelySquare: boolean;
  }

  const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");

  function getSpecificIcon(iconName: string): IconState {
    const icon = iconPackManager.getCustomIcon(iconName);
    if (icon) {
      return {
        src: (darkModeQuery.matches ? icon.dark : icon.light) || icon.base,
        mask: icon.mask,
        isAproximatelySquare: icon.isAproximatelySquare,
      };
    }
    return { src: null, mask: null, isAproximatelySquare: false };
  }

  let state = $state<IconState>(getSpecificIcon(name));
  let unlistener: UnlistenFn | null = null;

  function updateSrc(): void {
    state = getSpecificIcon(name);
  }

  // Watch for name changes
  $effect(() => {
    name; // track name dependency
    updateSrc();
  });

  onMount(async () => {
    darkModeQuery.addEventListener("change", updateSrc);
    unlistener = await iconPackManager.onChange(updateSrc);
  });

  onDestroy(() => {
    unlistener?.();
    unlistener = null;
    darkModeQuery.removeEventListener("change", updateSrc);
  });
</script>

{#if state.src}
  <figure {...imgProps} class={["slu-icon-outer", className]}>
    <img src={state.src} alt="" />
    {#if state.mask}
      <div
        class="slu-icon-mask"
        style="mask-image: url('{state.mask}')"
      ></div>
    {/if}
  </figure>
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
    background-color: var(--config-accent-light-color);
  }
</style>