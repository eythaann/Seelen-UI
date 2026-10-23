import type { NativeBlurShape } from "../../../gen/types/NativeBlurShape.ts";

type Radii = NativeBlurShape["radii"];

/** CSS Backgrounds §4.5: proportionally reduce overlapping corner radii. */
export function normalizeRadii(radii: Radii, width: number, height: number): Radii {
  const [tl, tr, br, bl] = radii;
  const factor = Math.min(
    1,
    width / (tl[0] + tr[0] || 1),
    width / (bl[0] + br[0] || 1),
    height / (tl[1] + bl[1] || 1),
    height / (tr[1] + br[1] || 1),
  );
  return radii.map(([x, y]) => [x * factor, y * factor]) as Radii;
}

export function resolveRadius(value: string, width: number, height: number): [number, number] {
  const parts = value.trim().split(/\s+/);
  const resolve = (part: string, size: number): number => {
    const number = Number.parseFloat(part);
    return Number.isFinite(number) ? Math.max(0, part.endsWith("%") ? number * size / 100 : number) : 0;
  };
  return [resolve(parts[0] ?? "0", width), resolve(parts[1] ?? parts[0] ?? "0", height)];
}

/** Read only axis-aligned geometry. Unsupported shapes are omitted, never overdrawn. */
export function readNativeBlurGeometry(
  element: HTMLElement,
  readStyle: (element: Element) => CSSStyleDeclaration = getComputedStyle,
): { clips: NativeBlurShape[]; opacity: number } | null {
  const bounds = element.getBoundingClientRect();
  if (bounds.width <= 0 || bounds.height <= 0) return null;
  const clips: NativeBlurShape[] = [];
  let opacity = 1;
  for (let node: HTMLElement | null = element; node; node = node.parentElement) {
    const style = readStyle(node);
    if (style.display === "none" || style.visibility !== "visible" || style.contentVisibility === "hidden") return null;
    if (
      style.clipPath !== "none" || style.maskImage !== "none" || style.filter !== "none" ||
      (style.clip && style.clip !== "auto") || style.perspective !== "none" ||
      (style.rotate !== "none" && style.rotate !== "0deg")
    ) return null;
    if (style.transform !== "none") {
      const matrix = new DOMMatrixReadOnly(style.transform);
      if (
        !matrix.is2D || Math.abs(matrix.b) > 0.00001 || Math.abs(matrix.c) > 0.00001 ||
        matrix.a <= 0 || matrix.d <= 0
      ) return null;
    }
    if (style.scale !== "none" && style.scale.split(/\s+/).some((value) => Number.parseFloat(value) <= 0)) return null;
    opacity *= Number.parseFloat(style.opacity);
    if (!Number.isFinite(opacity) || opacity <= 0) return null;
    const isElement = node === element;
    const clipsOverflow = [style.overflowX, style.overflowY].some((value) => value !== "visible") ||
      /(^|\s)(paint|strict|content)(\s|$)/.test(style.contain) || style.contentVisibility === "auto";
    if (!isElement && !clipsOverflow) continue;
    const rect = isElement ? bounds : node.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;
    const number = (value: string): number => Number.parseFloat(value) || 0;
    const localWidth = number(style.width) +
      (style.boxSizing === "border-box"
        ? 0
        : number(style.paddingLeft) + number(style.paddingRight) + number(style.borderLeftWidth) +
          number(style.borderRightWidth));
    const localHeight = number(style.height) +
      (style.boxSizing === "border-box"
        ? 0
        : number(style.paddingTop) + number(style.paddingBottom) + number(style.borderTopWidth) +
          number(style.borderBottomWidth));
    const width = localWidth || node.offsetWidth;
    const height = localHeight || node.offsetHeight;
    if (width <= 0 || height <= 0 || node.getClientRects().length !== 1) return null;
    const scaleX = rect.width / width;
    const scaleY = rect.height / height;
    const radii = normalizeRadii(
      [
        resolveRadius(style.borderTopLeftRadius, width, height),
        resolveRadius(style.borderTopRightRadius, width, height),
        resolveRadius(style.borderBottomRightRadius, width, height),
        resolveRadius(style.borderBottomLeftRadius, width, height),
      ],
      width,
      height,
    ).map(([x, y]) => [x * scaleX, y * scaleY]) as Radii;
    if (isElement) {
      clips.push({ x: rect.x, y: rect.y, width: rect.width, height: rect.height, radii });
    }
    if (clipsOverflow && !isElement) {
      // Scrollbars and borders are excluded from the ancestor padding-box clip.
      const left = node.clientLeft * scaleX;
      const top = node.clientTop * scaleY;
      const clipWidth = node.clientWidth * scaleX;
      const clipHeight = node.clientHeight * scaleY;
      const right = rect.width - left - clipWidth;
      const bottom = rect.height - top - clipHeight;
      const inner = radii.map(([x, y], index) => [
        Math.max(0, x - (index === 0 || index === 3 ? left : right)),
        Math.max(0, y - (index < 2 ? top : bottom)),
      ]) as Radii;
      if (clipWidth <= 0 || clipHeight <= 0) return null;
      clips.push({
        x: rect.x + left,
        y: rect.y + top,
        width: clipWidth,
        height: clipHeight,
        radii: normalizeRadii(inner, clipWidth, clipHeight),
      });
    }
    if (clips.length >= 16) return null;
  }
  clips.push({ x: 0, y: 0, width: innerWidth, height: innerHeight, radii: [[0, 0], [0, 0], [0, 0], [0, 0]] });
  const left = Math.max(...clips.map((clip) => clip.x));
  const top = Math.max(...clips.map((clip) => clip.y));
  const right = Math.min(...clips.map((clip) => clip.x + clip.width));
  const bottom = Math.min(...clips.map((clip) => clip.y + clip.height));
  return right > left && bottom > top ? { clips, opacity } : null;
}
