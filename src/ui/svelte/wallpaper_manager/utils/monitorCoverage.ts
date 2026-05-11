export type Rect = { left: number; top: number; right: number; bottom: number };

function mergedIntervalsHeight(intervals: [number, number][]): number {
  if (intervals.length === 0) return 0;
  intervals.sort((a, b) => a[0] - b[0]);
  let total = 0;
  let curStart = intervals[0]![0];
  let curEnd = intervals[0]![1];
  for (let i = 1; i < intervals.length; i++) {
    const start = intervals[i]![0];
    const end = intervals[i]![1];
    if (start <= curEnd) {
      curEnd = Math.max(curEnd, end);
    } else {
      total += curEnd - curStart;
      curStart = start;
      curEnd = end;
    }
  }
  return total + (curEnd - curStart);
}

/**
 * Returns the fraction [0, 1] of the monitor rect covered by the union of the given window rects.
 * Uses coordinate-compression sweep line for exact area union calculation.
 */
export function calculateMonitorCoverage(monitorRect: Rect, windowRects: Rect[]): number {
  const monitorWidth = monitorRect.right - monitorRect.left;
  const monitorHeight = monitorRect.bottom - monitorRect.top;
  const monitorArea = monitorWidth * monitorHeight;
  if (monitorArea <= 0) return 0;

  const clipped = windowRects
    .map((r) => ({
      left: Math.max(r.left, monitorRect.left),
      top: Math.max(r.top, monitorRect.top),
      right: Math.min(r.right, monitorRect.right),
      bottom: Math.min(r.bottom, monitorRect.bottom),
    }))
    .filter((r) => r.left < r.right && r.top < r.bottom);

  if (clipped.length === 0) return 0;

  const xs = [...new Set(clipped.flatMap((r) => [r.left, r.right]))].sort((a, b) => a - b);

  let coveredArea = 0;
  for (let i = 0; i < xs.length - 1; i++) {
    const x0 = xs[i] as number;
    const x1 = xs[i + 1] as number;
    const bandWidth = x1 - x0;
    const intervals = clipped
      .filter((r) => r.left <= x0 && r.right >= x1)
      .map((r): [number, number] => [r.top, r.bottom]);
    coveredArea += bandWidth * mergedIntervalsHeight(intervals);
  }

  return coveredArea / monitorArea;
}
