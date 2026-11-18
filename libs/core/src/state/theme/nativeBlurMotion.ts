import { cssEasing } from "./nativeBlurAnimation.ts";
import { normalizeRadii } from "./nativeBlurGeometry.ts";

type Point = [number, number];

/** The opt-in proxy uses a single two-keyframe translate/scale and uniform pixel radius. */
export function readNativeBlurMotion(
  element: HTMLElement,
  animation: Animation,
  durationMs: number,
): Point[][] | undefined {
  const effect = animation.effect as KeyframeEffect;
  const timing = effect.getTiming();
  const frames = effect.getKeyframes();
  const current = animation.currentTime;
  const duration = Number(timing.duration);
  const ease = cssEasing(timing.easing ?? "linear");
  const keyframeEase = cssEasing(frames[0]?.easing ?? "linear");
  if (
    frames.length !== 2 || typeof current !== "number" || !ease || !keyframeEase ||
    !Number.isFinite(duration) || duration <= 0 || animation.playbackRate !== 1 ||
    timing.direction !== "normal" || timing.iterations !== 1 ||
    frames[0]?.computedOffset !== 0 || frames[1]?.computedOffset !== 1
  ) return;

  for (let node: HTMLElement | null = element; node; node = node.parentElement) {
    const style = getComputedStyle(node);
    if (
      (node !== element && style.transform !== "none") || style.perspective !== "none" ||
      style.translate !== "none" || style.scale !== "none" || style.rotate !== "none"
    ) return;
  }
  const style = getComputedStyle(element);
  if (style.transformOrigin.split(/\s+/).some((value) => Number.parseFloat(value) !== 0)) return;
  const readTransform = (value: unknown): DOMMatrixReadOnly | undefined => {
    if (
      typeof value !== "string" ||
      !/^translate3d\([^()]+\) scale\([^()]+\)$/.test(value)
    ) return;
    const matrix = new DOMMatrixReadOnly(value);
    if (matrix.m43 !== 0 || matrix.a <= 0 || matrix.d <= 0) return;
    return matrix;
  };
  const from = readTransform(frames[0]?.transform), to = readTransform(frames[1]?.transform);
  const radius = frames.map(({ borderRadius }) =>
    typeof borderRadius === "string" && /^\d+(\.\d+)?px$/.test(borderRadius)
      ? Number.parseFloat(borderRadius)
      : Number.NaN
  );
  if (!from || !to || !radius.every(Number.isFinite)) return;
  const actual = new DOMMatrixReadOnly(style.transform);
  if (actual.m43 !== 0 || actual.a <= 0 || actual.d <= 0) return;
  const rect = element.getBoundingClientRect();
  const width = rect.width / actual.a, height = rect.height / actual.d;
  const left = rect.left - actual.e, top = rect.top - actual.f;
  if (width <= 0 || height <= 0) return;

  const steps = Math.min(31, Math.max(1, Math.ceil(durationMs / 8)));
  const arcSteps = 15;
  const outlines: Point[][] = [];
  for (let i = 0; i <= steps; i++) {
    const time = current + i * durationMs / steps - (timing.delay ?? 0);
    const progress = keyframeEase(ease(Math.max(0, Math.min(1, time / duration))));
    const mix = (a: number, b: number) => a + (b - a) * progress;
    const x = left + mix(from.e, to.e), y = top + mix(from.f, to.f);
    const sx = mix(from.a, to.a), sy = mix(from.d, to.d);
    const r = mix(radius[0]!, radius[1]!);
    const [[rx, ry]] = normalizeRadii([[r, r], [r, r], [r, r], [r, r]], width, height);
    const centers = [[rx, ry], [width - rx, ry], [width - rx, height - ry], [rx, height - ry]];
    const points: Point[] = [];
    for (const [corner, [cx, cy]] of centers.entries()) {
      for (let point = 0; point <= arcSteps; point++) {
        const angle = Math.PI * (1 + corner / 2 + point / (2 * arcSteps));
        points.push([x + (cx! + rx * Math.cos(angle)) * sx, y + (cy! + ry * Math.sin(angle)) * sy]);
      }
    }
    outlines.push(points);
  }
  return outlines;
}
