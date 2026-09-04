<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import { iconPackManager, type IconState } from "./common.svelte.ts";
  import { prefersDarkColorScheme } from "../../runes/DarkMode.svelte.ts";
  import SluIconRenderer from "./SluIconRenderer.svelte";

  interface Props {
    class?: ClassValue;
    [key: string]: any;
  }

  let { class: className, ...rest }: Props = $props();

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

<SluIconRenderer {...rest} {state} class={className} />
