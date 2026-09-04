<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import type { SeelenCommandGetIconArgs } from "@seelen-ui/lib/types";
  import { IconPackManager } from "@seelen-ui/lib";
  import { iconPackManager, type IconState } from "./common.svelte.ts";
  import MissingIcon from "./MissingIcon.svelte";
  import SluIconRenderer from "./SluIconRenderer.svelte";
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
  <SluIconRenderer
    {...imgProps}
    state={icon}
    class={className}
    {lazy}
    data-path={path ?? undefined}
    data-umid={umid ?? undefined}
  />
{:else}
  <MissingIcon {...imgProps} class={className} />
{/if}
