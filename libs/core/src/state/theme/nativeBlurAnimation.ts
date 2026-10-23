/** Limited CSS easing bridge. Unsupported easing stays on the snapshot fallback. */
export function cssEasing(value: string): ((progress: number) => number) | null {
  if (value === "linear") return (p) => p;
  const presets: Record<string, number[]> = {
    ease: [.25, .1, .25, 1],
    "ease-in": [.42, 0, 1, 1],
    "ease-out": [0, 0, .58, 1],
    "ease-in-out": [.42, 0, .58, 1],
  };
  const match = /^cubic-bezier\(([^)]+)\)$/.exec(value);
  const points = presets[value] ?? match?.[1]?.split(",").map(Number);
  if (!points || points.length !== 4 || !points.every(Number.isFinite)) return null;
  const [x1, y1, x2, y2] = points as [number, number, number, number];
  if (x1 < 0 || x1 > 1 || x2 < 0 || x2 > 1) return null;
  const curve = (t: number, a: number, b: number): number =>
    3 * (1 - t) ** 2 * t * a + 3 * (1 - t) * t ** 2 * b + t ** 3;
  return (p) => {
    if (p <= 0) return 0;
    if (p >= 1) return 1;
    let low = 0, high = 1;
    for (let i = 0; i < 20; i++) {
      const middle = (low + high) / 2;
      if (curve(middle, x1, x2) < p) low = middle;
      else high = middle;
    }
    return curve((low + high) / 2, y1, y2);
  };
}

interface Track {
  key: string;
  remaining: number;
  at: (elapsed: number) => number;
}

export interface OpacityPlan {
  key: string;
  startedAtMs: number;
  durationMs: number;
  values: number[];
}

/** Predict only simple replace-opacity animations; never reinterpret complex CSS. */
export function readOpacityPlan(
  element: HTMLElement,
  previous?: OpacityPlan,
  readStyle: (element: Element) => CSSStyleDeclaration = getComputedStyle,
): OpacityPlan | undefined {
  const tracks: Track[] = [];
  let base = 1;
  for (let node: HTMLElement | null = element; node; node = node.parentElement) {
    const animations = node.getAnimations().filter((a) =>
      (a.effect as KeyframeEffect | null)?.target === node &&
      (a.effect as KeyframeEffect).getKeyframes().some((frame) => frame.opacity !== undefined)
    );
    if (!animations.length) {
      base *= Number(readStyle(node).opacity);
      continue;
    }
    if (animations.length !== 1) return;
    const animation = animations[0]!;
    const effect = animation.effect as KeyframeEffect;
    const timing = effect.getTiming();
    const frames = effect.getKeyframes();
    const duration = Number(timing.duration);
    const current = animation.currentTime;
    if (
      animation.playState !== "running" || typeof current !== "number" ||
      animation.playbackRate <= 0 || timing.iterations !== 1 || timing.direction !== "normal" ||
      (timing.iterationStart ?? 0) !== 0 ||
      (timing.endDelay ?? 0) !== 0 ||
      ("timeline" in animation && animation.timeline !== document.timeline) ||
      effect.composite !== "replace" || frames.length !== 2 ||
      frames.some((frame) => frame.composite && frame.composite !== "auto" && frame.composite !== "replace") ||
      frames[0]!.computedOffset !== 0 || frames[1]!.computedOffset !== 1 ||
      !Number.isFinite(duration) || duration <= 0
    ) return;
    const from = Number(frames[0]!.opacity), to = Number(frames[1]!.opacity);
    const outer = cssEasing(timing.easing ?? "linear");
    const inner = cssEasing(frames[0]!.easing ?? "linear");
    if (!outer || !inner || !Number.isFinite(from) || !Number.isFinite(to)) return;
    const delay = timing.delay ?? 0;
    const rate = animation.playbackRate;
    tracks.push({
      key: JSON.stringify([animation.startTime, rate, duration, delay, from, to, timing.easing, frames[0]!.easing]),
      remaining: (delay + duration - current) / rate,
      at: (elapsed) =>
        Math.max(
          0,
          Math.min(
            1,
            from + (to - from) * inner(outer(Math.max(0, Math.min(1, (current + elapsed * rate - delay) / duration)))),
          ),
        ),
    });
  }
  if (!tracks.length) return;
  const durationMs = Math.max(...tracks.map((track) => track.remaining));
  if (durationMs <= 0 || durationMs > 2000 || !Number.isFinite(base)) return;
  const key = JSON.stringify([base, tracks.map((track) => track.key)]);
  if (previous?.key === key) return previous;
  const count = Math.min(128, Math.max(2, Math.ceil(durationMs / 8) + 1));
  return {
    key,
    startedAtMs: performance.timeOrigin + performance.now(),
    durationMs,
    values: Array.from(
      { length: count },
      (_, i) =>
        Math.max(
          0,
          Math.min(1, base * tracks.reduce((opacity, track) => opacity * track.at(i * durationMs / (count - 1)), 1)),
        ),
    ),
  };
}
