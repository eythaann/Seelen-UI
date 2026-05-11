<script lang="ts">
  import { state } from "../../../state.svelte.ts";
  import ReservedContainer from './Reserved.svelte';

  interface Props {
    hwnd: number;
    growFactor?: number;
  }

  let { hwnd, growFactor }: Props = $props();

  let isFocused = $derived(state.focusedApp.hwnd === hwnd);
</script>

<div
  data-hwnd={hwnd}
  style:flex-grow={growFactor}
  class="wm-container wm-leaf"
  class:wm-leaf-focused={isFocused}
  class:wm-leaf-with-borders={state.settings.border.enabled}
>
  {#if state.reservation && isFocused}
    <ReservedContainer reservation={state.reservation} />
  {/if}
</div>
