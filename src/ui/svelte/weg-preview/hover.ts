export interface PreviewRect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** Physical screen coordinates, including the short gap between the dock item and popup. */
export function insidePreviewBridge(point: number[], popup: PreviewRect, source: PreviewRect): boolean {
  const [x, y] = point;
  if (x === undefined || y === undefined) return false;
  return x >= Math.min(popup.left, source.left) && x <= Math.max(popup.right, source.right) &&
    y >= Math.min(popup.top, source.top) && y <= Math.max(popup.bottom, source.bottom);
}
