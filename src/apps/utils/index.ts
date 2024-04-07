export function toPhysicalPixels(size: number): number {
  return Math.floor(size * window.devicePixelRatio);
}