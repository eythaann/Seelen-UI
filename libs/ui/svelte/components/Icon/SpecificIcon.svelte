<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import { iconPackManager, type IconState } from "./common.svelte.ts";
  import { prefersDarkColorScheme } from "../../runes/DarkMode.svelte.ts";
  import MissingIcon from "./MissingIcon.svelte";
  import SluIconRenderer from "./SluIconRenderer.svelte";

  interface Props {
    name: string;
    class?: ClassValue;
    lazy?: boolean;
    [key: string]: any;
  }

  let { name, class: className, lazy, ...imgProps }: Props = $props();

  let state: IconState = $derived.by(() => {
    // Depend on _version to trigger reactivity when icon pack changes
    iconPackManager._version;
    const icon = iconPackManager.value.getCustomIcon(name);
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
  <SluIconRenderer {...imgProps} {state} class={className} {lazy} />
{:else}
  <MissingIcon {...imgProps} class={className} />
{/if}
