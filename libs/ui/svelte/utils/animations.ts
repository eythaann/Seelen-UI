import type { TransitionConfig } from "svelte/transition";

interface Params {
  duration?: number;
}

interface Options {
  direction: "in" | "out" | "both";
}

type TransitionGenerator = (opt: { direction: "in" | "out" }) => TransitionConfig;
type TransitionSetup = (
  node: HTMLElement,
  params: Params,
  opts: Options,
) => TransitionConfig | TransitionGenerator;

const _CssHandled: TransitionSetup = (node, { duration = 200 } = {}) => {
  return ({ direction }) => {
    if (direction === "in") {
      delete node.dataset.unmounting;
      node.dataset.mounting = "";
      return {
        duration,
        tick: (t) => {
          node.toggleAttribute("data-mounting", t < 1);
        },
      };
    }

    delete node.dataset.mounting;
    node.dataset.unmounting = "";
    return {
      duration,
    };
  };
};

// svelte-check's bundled shims (svelte-shims-v4.d.ts) still type a transition's returned
// generator as `() => TransitionConfig`, predating https://github.com/sveltejs/svelte/pull/8318
// which lets it receive `{ direction }`. Svelte itself supports this at runtime, so this cast
// only appeases the outdated checker.
export const CssHandled = _CssHandled as unknown as (
  node: HTMLElement,
  params?: Params,
) => TransitionConfig | (() => TransitionConfig);
