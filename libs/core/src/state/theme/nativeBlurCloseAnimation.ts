import type { NativeBlurShape } from "../../../gen/types/NativeBlurShape.ts";
import { cssEasing } from "./nativeBlurAnimation.ts";
import { projectFlipOutline } from "./nativeBlurProjection.ts";

/** Predict the opted-in, initially flat perspective/rotateX transition. */
export function readCloseOutlines(
  element: HTMLElement,
  flat: NativeBlurShape,
  durationMs: number,
): [number, number][][] | undefined {
  const style = getComputedStyle(element);
  if (style.translate !== "none" || style.scale !== "none") return;
  for (let ancestor = element.parentElement; ancestor; ancestor = ancestor.parentElement) {
    const parent = getComputedStyle(ancestor);
    if (parent.transform !== "none" || parent.translate !== "none" || parent.scale !== "none") return;
  }
  const transform = element.getAnimations().find((animation) =>
    animation instanceof CSSTransition && animation.transitionProperty === "transform"
  );
  if (!transform || typeof transform.currentTime !== "number" || transform.playbackRate !== 1) return;
  const effect = transform.effect as KeyframeEffect;
  const timing = effect.getTiming();
  const frames = effect.getKeyframes();
  const duration = Number(timing.duration);
  const current = transform.currentTime - (timing.delay ?? 0);
  const easing = cssEasing(timing.easing ?? "linear");
  const keyframeEasing = cssEasing(frames[0]?.easing ?? "linear");
  if (
    !easing || !keyframeEasing || frames.length !== 2 || duration <= 0 || !Number.isFinite(duration) ||
    !frames[0]?.transform || !frames[1]?.transform
  ) return;
  const start = new DOMMatrixReadOnly(String(frames[0].transform));
  const end = new DOMMatrixReadOnly(String(frames[1].transform));
  const angle = Math.atan2(end.m23, end.m22);
  const depth = Math.hypot(end.m24, end.m34);
  if (Math.abs(angle) < 0.001 || Math.abs(angle) > Math.PI / 2 + 0.00001 || depth < 0.00001) return;
  const matrix = (theta: number) => {
    const c = Math.cos(theta), s = Math.sin(theta);
    return [1, 0, 0, 0, 0, c, s, -s * depth, 0, -s, c, -c * depth, 0, 0, 0, 1];
  };
  const matches = (value: DOMMatrixReadOnly, expected: number[]) =>
    Array.from(value.toFloat64Array()).every((component, i) => Math.abs(component - expected[i]!) < 0.00002);
  if ((!start.isIdentity && !matches(start, matrix(0))) || !matches(end, matrix(angle))) return;
  const [originX, originY, originZ = 0] = style.transformOrigin.split(/\s+/).map(Number.parseFloat);
  if (originZ !== 0 || !Number.isFinite(originX) || !Number.isFinite(originY)) return;

  const outlines: [number, number][][] = [];
  const steps = 12;
  for (let i = 0; i <= steps; i++) {
    const progress = Math.max(0, Math.min(1, (current + durationMs * i / steps) / duration));
    const theta = Math.max(-1.56, Math.min(1.56, angle * keyframeEasing(easing(progress))));
    const points = projectFlipOutline(
      flat.width,
      flat.height,
      flat.radii,
      matrix(theta),
      [originX!, originY!],
      flat,
      true,
    );
    if (!points || (outlines.length && points.length !== outlines[0]!.length)) return;
    outlines.push(points);
  }
  return outlines;
}
