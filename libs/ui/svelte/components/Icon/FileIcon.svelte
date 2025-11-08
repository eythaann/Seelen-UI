<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { ClassValue } from "svelte/elements";
  import type { SeelenCommandGetIconArgs } from "@seelen-ui/lib/types";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { IconPackManager } from "@seelen-ui/lib";
  import { iconPackManager } from "./common.ts";
  import MissingIcon from "./MissingIcon.svelte";

  interface Props extends SeelenCommandGetIconArgs {
    /** if true, no missing icon will be rendered in case no icon found */
    noFallback?: boolean;
    class?: ClassValue;
    [key: string]: any;
  }

  let { path, umid, noFallback = false, class: className, ...imgProps }: Props = $props();

  interface IconState {
    src: string | null;
    mask: string | null;
    isAproximatelySquare: boolean;
  }

  const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");

  function getIcon(args: SeelenCommandGetIconArgs): IconState {
    const icon = iconPackManager.getIcon(args);
    if (icon) {
      return {
        src: (darkModeQuery.matches ? icon.dark : icon.light) || icon.base,
        mask: icon.mask,
        isAproximatelySquare: icon.isAproximatelySquare,
      };
    }
    return { src: null, mask: null, isAproximatelySquare: false };
  }

  let state = $state<IconState>(getIcon({ path, umid }));
  let unlistener: UnlistenFn | null = null;
  let previousSrc: string | null = null;

  function requestIconExtraction(): void {
    IconPackManager.requestIconExtraction({ path, umid });
  }

  function updateSrc(): void {
    state = getIcon({ path, umid });
  }

  // Watch for path/umid changes
  $effect(() => {
    path;
    umid;
    updateSrc();
  });

  // Watch for src becoming null (trigger icon extraction)
  $effect(() => {
    if (previousSrc && !state.src) {
      requestIconExtraction();
    }
    previousSrc = state.src;
  });

  onMount(async () => {
    darkModeQuery.addEventListener("change", updateSrc);
    unlistener = await iconPackManager.onChange(updateSrc);

    // Initial extraction request if no icon found
    if (!state.src) {
      requestIconExtraction();
    }
  });

  onDestroy(() => {
    unlistener?.();
    unlistener = null;
    darkModeQuery.removeEventListener("change", updateSrc);
  });

  const dataProps = $derived(
    Object.entries(imgProps)
      .filter(([k]) => k.startsWith("data-"))
      .reduce((acc, [k, v]) => ({ ...acc, [k]: v }), {})
  );
</script>

{#if state.src}
  <figure
    {...imgProps}
    class={["slu-icon-outer", className]}
    data-shape={state.isAproximatelySquare ? "square" : "unknown"}
  >
    <img {...dataProps} src={state.src} alt="" />
    {#if state.mask}
      <div
        {...dataProps}
        class="slu-icon-mask sl-mask"
        style="mask-image: url('{state.mask}')"
      ></div>
    {/if}
  </figure>
{:else if !noFallback}
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
    background-color: var(--config-accent-light-color);
  }
</style>