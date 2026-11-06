/// <reference lib="deno.ns" />

import { assert, assertAlmostEquals, assertEquals, assertStrictEquals } from "@std/assert";
import { cssEasing, readOpacityPlan } from "./nativeBlurAnimation.ts";

Deno.test("native opacity bridge preserves supported CSS easing and rejects unsupported curves", () => {
  assertEquals(cssEasing("steps(2, end)"), null);
  assertEquals(cssEasing("cubic-bezier(2, 0, 1, 1)"), null);
  assertEquals(cssEasing("ease-in")!(0), 0);
  assertEquals(cssEasing("ease-in")!(1), 1);
  assertAlmostEquals(cssEasing("ease-in-out")!(0.5), 0.5, 0.00001);
  assert(cssEasing("ease-in")!(0.5) < 0.5);
});

Deno.test("native opacity bridge samples remaining fade and multiplies ancestor opacity", () => {
  const previous = globalThis.getComputedStyle;
  globalThis.getComputedStyle = (() => ({ opacity: "0.5" })) as unknown as typeof getComputedStyle;
  try {
    const parent = { parentElement: null, getAnimations: () => [] };
    const element = { parentElement: parent, getAnimations: () => [animation] };
    const animation = {
      currentTime: 25,
      startTime: 100,
      playbackRate: 1,
      playState: "running",
      effect: {
        target: element,
        composite: "replace",
        getTiming: () => ({ duration: 100, delay: 0, iterations: 1, direction: "normal", easing: "linear" }),
        getKeyframes: () => [
          { computedOffset: 0, opacity: "1", easing: "linear" },
          { computedOffset: 1, opacity: "0", easing: "linear" },
        ],
      },
    };
    const plan = readOpacityPlan(element as unknown as HTMLElement)!;
    assertEquals(plan.durationMs, 75);
    assertAlmostEquals(plan.values[0]!, 0.375);
    assertEquals(plan.values.at(-1), 0);
    animation.currentTime = 50;
    assertEquals(readOpacityPlan(element as unknown as HTMLElement)!.key, plan.key);
    assertStrictEquals(readOpacityPlan(element as unknown as HTMLElement, plan), plan);
    animation.playbackRate = -1;
    assertEquals(readOpacityPlan(element as unknown as HTMLElement), undefined);
    animation.playbackRate = 1;
    animation.playState = "paused";
    assertEquals(readOpacityPlan(element as unknown as HTMLElement), undefined);
  } finally {
    globalThis.getComputedStyle = previous;
  }
});
