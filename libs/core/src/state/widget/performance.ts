import type { PerformanceMode } from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "../../handlers/mod.ts";
import { RuntimeStyleSheet } from "../../utils/DOM.ts";

export async function disableAnimationsOnPerformanceMode(): Promise<void> {
  const initial = await invoke(SeelenCommand.StateGetPerformanceMode);
  setDisableAnimations(initial);
  subscribe(SeelenEvent.StatePerformanceModeChanged, (e) => {
    setDisableAnimations(e.payload);
  });
}

function setDisableAnimations(mode: PerformanceMode): void {
  document.documentElement.dataset.performanceMode = mode;
}

/** Matches the root itself and everything under it (a nested `*` would skip the root). */
const withDescendants = (root: string) => `${root}, ${root} *, ${root} *::before, ${root} *::after`;

const DISABLED_ANIMATIONS_CSS = `
  animation-duration: 0.001ms !important;
  animation-iteration-count: 1 !important;
  animation-delay: 0s !important;

  transition-duration: 0.001ms !important;
  transition-delay: 0s !important;

  scroll-behavior: auto !important;
`;

// Used by performance mode to reduce cpu/gpu usage
const PERF_CSS = `
${withDescendants("html:not([data-widget-ready])")} {
  ${DISABLED_ANIMATIONS_CSS}
}

${withDescendants(`html[data-performance-mode="Extreme"]`)} {
  ${DISABLED_ANIMATIONS_CSS}
}

@media (prefers-reduced-motion: reduce) {
  ${DISABLED_ANIMATIONS_CSS}
}
`;

export function registerPerformanceStyles(): void {
  const styleSheet = new RuntimeStyleSheet("@static/performance");
  styleSheet.addStyle(PERF_CSS);
  styleSheet.applyToDocument();
}
