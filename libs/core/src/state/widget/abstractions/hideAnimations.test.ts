/// <reference lib="deno.ns" />
import { assertEquals } from "@std/assert";
import {
  captureHideAnimations,
  collectHideAnimations,
  hideAnimationBudget,
  waitHideAnimations,
} from "./hideAnimations.ts";

function animation(endTime: number, iterations = 1): Animation {
  return {
    currentTime: 0,
    playbackRate: 1,
    playState: "running",
    effect: {
      getComputedTiming: () => ({ endTime, iterations }),
      getTiming: () => ({ duration: endTime, iterations }),
    },
    finished: Promise.resolve(),
  } as unknown as Animation;
}

Deno.test("popup hide ignores pre-existing content animations and infinite loops", () => {
  const content = animation(5000), loop = animation(Infinity, Infinity), close = animation(85);
  assertEquals(collectHideAnimations(captureHideAnimations([content]), [content, loop, close]), [close]);
  assertEquals(hideAnimationBudget([close], 200), 200);
  assertEquals(hideAnimationBudget([content], 200), 200);
  assertEquals(hideAnimationBudget([], 200), 0);
});

Deno.test("popup hide recognizes an existing animation resumed by the visibility style", () => {
  const close = animation(85);
  Object.defineProperty(close, "playState", { value: "paused", configurable: true });
  const before = captureHideAnimations([close]);
  Object.defineProperty(close, "playState", { value: "running" });
  assertEquals(collectHideAnimations(before, [close]), [close]);
});

Deno.test("popup hide also completes when its transition is cancelled", async () => {
  const close = animation(85);
  Object.defineProperty(close, "finished", { value: Promise.reject(new Error("cancelled")) });
  await waitHideAnimations([close], 85);
});

Deno.test("popup hide waits for completion even when the CSS timeline trails its nominal duration", async () => {
  const close = animation(5);
  let completed = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  Object.defineProperty(close, "finished", {
    value: new Promise<void>((resolve) => {
      timer = setTimeout(() => {
        completed = true;
        resolve();
      }, 25);
    }),
  });
  try {
    await waitHideAnimations([close], hideAnimationBudget([close], 200));
    assertEquals(completed, true);
  } finally {
    clearTimeout(timer);
  }
});

Deno.test("popup hide keeps a watchdog for animations that never finish", async () => {
  const close = animation(1000);
  Object.defineProperty(close, "finished", { value: new Promise<void>(() => {}) });
  await waitHideAnimations([close], 10);
});
