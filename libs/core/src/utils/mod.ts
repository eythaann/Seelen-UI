export class Rect {
  left = 0;
  top = 0;
  right = 0;
  bottom = 0;
}

export function isSeelenUIRuntime(): boolean {
  return !!(globalThis.window as any).__SLU_WIDGET;
}

export { RuntimeStyleSheet } from "./DOM.ts";
