<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import type { IconState } from "./common.svelte.ts";

  interface Props {
    state: IconState;
    class?: ClassValue;
    lazy?: boolean;
    [key: string]: any;
  }

  let { state, class: className, lazy, ...rest }: Props = $props();
</script>

<figure
  {...rest}
  class={["slu-icon-outer", className]}
  data-shape={state.isAproximatelySquare ? "square" : "unknown"}
>
  <img src={state.src || ""} alt="" loading={lazy ? "lazy" : "eager"} draggable="false" />
  {#if state.mask}
    <div class="slu-icon-mask" style="mask-image: url('{state.mask}')"></div>
  {/if}
</figure>

<style>
  .slu-icon-outer {
    position: relative;

    img {
      width: 100%;
      height: 100%;
      object-fit: contain;
    }
  }

  .slu-icon-mask {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    mask-repeat: no-repeat;
    mask-size: contain;
    mask-position: center;
    mask-mode: luminance;
    background-color: var(--system-accent-color);
    mix-blend-mode: multiply;
  }
</style>
