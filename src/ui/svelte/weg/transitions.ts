import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

let isInitialLoad = true;
if (typeof window !== "undefined") {
  setTimeout(() => {
    isInitialLoad = false;
  }, 600);
}

export function dockItemExit(
  node: HTMLElement,
  { horizontal = true }: { horizontal?: boolean } = {},
): TransitionConfig {
  const style = getComputedStyle(node);
  const targetOpacity = +style.opacity || 1;
  const targetDimension = horizontal ? node.offsetWidth : node.offsetHeight;
  const fallbackSize = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-item-size")) ||
    48;
  const dimension = targetDimension > 0 ? targetDimension : fallbackSize;
  const gap = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-space-between-items")) ||
    8;

  const halfGap = gap / 2;

  return {
    duration: 240,
    easing: cubicOut,
    css: (t: number) => {
      const scale = t;
      const currentSize = dimension * t;
      const currentMargin = (1 - t) * -halfGap;
      const sizeStyle = horizontal
        ? `max-width: ${currentSize}px; width: ${currentSize}px; min-width: 0px; margin-left: ${currentMargin}px; margin-right: ${currentMargin}px;`
        : `max-height: ${currentSize}px; height: ${currentSize}px; min-height: 0px; margin-top: ${currentMargin}px; margin-bottom: ${currentMargin}px;`;

      return `opacity: ${
        t * targetOpacity
      }; transform: scale(${scale}); transform-origin: center center; ${sizeStyle} display: flex; justify-content: center; align-items: center; flex-shrink: 0;`;
    },
  };
}

export function dockItemEnter(
  node: HTMLElement,
  { horizontal = true }: { horizontal?: boolean } = {},
): TransitionConfig {
  if (isInitialLoad) {
    return { duration: 0 };
  }

  const style = getComputedStyle(node);
  const targetOpacity = +style.opacity || 1;
  const targetDimension = horizontal ? node.offsetWidth : node.offsetHeight;
  const fallbackSize = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-item-size")) ||
    48;
  const dimension = targetDimension > 0 ? targetDimension : fallbackSize;
  const gap = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--config-space-between-items")) ||
    8;
  const halfGap = gap / 2;

  return {
    duration: 240,
    easing: cubicOut,
    css: (t: number) => {
      const scale = t;
      const currentSize = dimension * t;
      const currentMargin = (1 - t) * -halfGap;
      const sizeStyle = horizontal
        ? `max-width: ${currentSize}px; width: ${currentSize}px; min-width: 0px; margin-left: ${currentMargin}px; margin-right: ${currentMargin}px;`
        : `max-height: ${currentSize}px; height: ${currentSize}px; min-height: 0px; margin-top: ${currentMargin}px; margin-bottom: ${currentMargin}px;`;

      return `opacity: ${
        t * targetOpacity
      }; transform: scale(${scale}); transform-origin: center center; ${sizeStyle} display: flex; justify-content: center; align-items: center; flex-shrink: 0;`;
    },
  };
}
