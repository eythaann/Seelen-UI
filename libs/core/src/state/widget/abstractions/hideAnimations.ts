interface AnimationSnapshot {
  playState: AnimationPlayState;
  startTime: Animation["startTime"];
  timing: string;
}

function snapshot(animation: Animation): AnimationSnapshot {
  return {
    playState: animation.playState,
    startTime: animation.startTime,
    timing: JSON.stringify(animation.effect?.getTiming()),
  };
}

export function captureHideAnimations(animations: Animation[]): Map<Animation, AnimationSnapshot> {
  return new Map(animations.map((animation) => [animation, snapshot(animation)]));
}

/** Include new/resumed/retimed closing animations, not pre-existing content motion. */
export function collectHideAnimations(before: Map<Animation, AnimationSnapshot>, after: Animation[]): Animation[] {
  return after.filter((animation) => {
    if (animation.playState === "finished" || animation.effect?.getComputedTiming().iterations === Infinity) {
      return false;
    }
    const old = before.get(animation);
    if (!old) return true;
    const current = snapshot(animation);
    return old.playState !== current.playState || old.startTime !== current.startTime || old.timing !== current.timing;
  });
}

/** A watchdog, not an estimated finish time: CSS timelines can lag wall-clock timers. */
export function hideAnimationBudget(animations: Animation[], maximum: number): number {
  return animations.length ? maximum : 0;
}

export async function waitHideAnimations(animations: Animation[], budget: number): Promise<void> {
  if (!animations.length || budget <= 0) return;
  let timeout: ReturnType<typeof setTimeout> | undefined;
  try {
    await Promise.race([
      Promise.allSettled(animations.map((animation) => animation.finished)),
      new Promise<void>((resolve) => {
        timeout = setTimeout(resolve, budget);
      }),
    ]);
  } finally {
    clearTimeout(timeout);
  }
}
