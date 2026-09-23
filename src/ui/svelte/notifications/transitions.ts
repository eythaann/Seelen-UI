import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

export function notificationCardExit(node: HTMLElement): TransitionConfig {
  const style = getComputedStyle(node);
  const targetOpacity = +style.opacity || 1;
  const height = node.offsetHeight;
  const parent = node.parentElement;
  const gap = parent ? parseFloat(getComputedStyle(parent).rowGap || getComputedStyle(parent).gap) || 8 : 8;

  return {
    duration: 220,
    easing: cubicOut,
    css: (t: number) => {
      const u = 1 - t;
      const currentHeight = height * t;
      const currentMarginBottom = -gap * u;
      const swipeX = u * 28;
      const scale = 0.96 + 0.04 * t;

      return `opacity: ${
        t * targetOpacity
      }; transform: translateX(${swipeX}px) scale(${scale}); max-height: ${currentHeight}px; height: ${currentHeight}px; min-height: 0px; margin-bottom: ${currentMarginBottom}px; overflow: hidden; pointer-events: none;`;
    },
  };
}
