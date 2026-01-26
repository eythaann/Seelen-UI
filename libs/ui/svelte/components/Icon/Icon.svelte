<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import InlineSVG from "./InlineSVG.svelte";
  import type { IconName } from "libs/ui/icons";

  interface Props {
    iconName: IconName;
    size?: string | number;
    color?: string;
    class?: ClassValue;
    [key: string]: any;
  }

  let { iconName, size, color, class: className, ...rest }: Props = $props();

  const computedStyle = $derived.by(() => {
    const styles: string[] = [];
    if (size) {
      const sizeValue = typeof size === "number" ? `${size}px` : size;
      styles.push(`height: ${sizeValue}`);
    }
    if (color) {
      styles.push(`color: ${color}`);
    }
    return styles.join("; ");
  });
</script>

<InlineSVG
  {...rest}
  src={`/icons/${iconName}.svg`}
  class={["slu-icon", className]}
  style={computedStyle}
/>

<style>
  :global(.slu-icon) {
    height: 1rem;
    width: max-content;
    min-width: max-content;
    display: inline-block;

    > :global(svg) {
      vertical-align: middle;
    }
  }
</style>
