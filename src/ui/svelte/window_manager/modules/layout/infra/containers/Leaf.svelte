<script lang="ts">
  import { state } from "../../../shared/state.svelte.ts";
  // import { ReservedContainer } from './reserved';

  interface Props {
    hwnd: number;
    growFactor?: number;
  }

  let { hwnd, growFactor }: Props = $props();

  // Svelte 5 runes: $derived for computed values
  let isFocused = $derived(state.focusedApp.hwnd === hwnd);
</script>

<div
  data-hwnd={hwnd}
  style:flex-grow={growFactor}
  class="wm-container wm-leaf"
  class:wm-leaf-focused={isFocused}
  class:wm-leaf-with-borders={state.settings.border.enabled}
>
  <!-- {#if reservation && isFocused}
    <ReservedContainer {reservation} />
  {/if} -->
</div>
