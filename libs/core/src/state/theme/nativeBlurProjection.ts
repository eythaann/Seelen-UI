type Point = [number, number];
type Radii = [Point, Point, Point, Point];
interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Limited single-plane perspective/rotateX projection; no transformed ancestors. */
export function projectFlipOutline(
  width: number,
  height: number,
  radii: Radii,
  matrix: number[],
  origin: Point,
  bounds: Bounds,
  layoutCoordinates = false,
): Point[] | null {
  if (
    matrix.length !== 16 || !matrix.every(Number.isFinite) || width <= 0 || height <= 0 ||
    bounds.width <= 0 || bounds.height <= 0 ||
    ![...origin, bounds.x, bounds.y, bounds.width, bounds.height].every(Number.isFinite)
  ) return null;
  // X-axis hinge only. Reject arbitrary rotations, shear, negative scale and back faces.
  if ([1, 2, 3, 4, 8].some((i) => Math.abs(matrix[i]!) > 1e-6) || matrix[0]! <= 0 || matrix[5]! <= 1e-5) return null;
  const project = (x: number, y: number): Point | null => {
    const dx = x - origin[0], dy = y - origin[1];
    const w = matrix[3]! * dx + matrix[7]! * dy + matrix[15]!;
    if (!Number.isFinite(w) || w < 0.05) return null;
    const point: Point = [
      (matrix[0]! * dx + matrix[4]! * dy + matrix[12]!) / w + origin[0],
      (matrix[1]! * dx + matrix[5]! * dy + matrix[13]!) / w + origin[1],
    ];
    return point.every(Number.isFinite) ? point : null;
  };
  const corners = [[0, 0], [width, 0], [width, height], [0, height]].map(([x, y]) => project(x!, y!));
  if (corners.some((p) => !p)) return null;
  const minX = Math.min(...corners.map((p) => p![0])), maxX = Math.max(...corners.map((p) => p![0]));
  const minY = Math.min(...corners.map((p) => p![1])), maxY = Math.max(...corners.map((p) => p![1]));
  if (maxX - minX < 0.01 || maxY - minY < 0.01) return null;
  const sx = layoutCoordinates ? 1 : bounds.width / (maxX - minX);
  const sy = layoutCoordinates ? 1 : bounds.height / (maxY - minY);
  const offsetX = layoutCoordinates ? 0 : minX, offsetY = layoutCoordinates ? 0 : minY;
  const [tl, tr, br, bl] = radii;
  const arcs = [
    [tl[0], tl[1], tl, Math.PI],
    [width - tr[0], tr[1], tr, 1.5 * Math.PI],
    [width - br[0], height - br[1], br, 0],
    [bl[0], height - bl[1], bl, 0.5 * Math.PI],
  ] as const;
  const points: Point[] = [];
  for (const [cx, cy, radius, start] of arcs) {
    for (let step = 0; step <= 8; step++) {
      const angle = start + Math.PI * step / 16;
      const p = project(cx + radius[0] * Math.cos(angle), cy + radius[1] * Math.sin(angle));
      if (!p) return null;
      const point: Point = [bounds.x + (p[0] - offsetX) * sx, bounds.y + (p[1] - offsetY) * sy];
      const previous = points.at(-1);
      if (!previous || Math.hypot(point[0] - previous[0], point[1] - previous[1]) > 1e-5) points.push(point);
    }
  }
  if (points.length > 1 && Math.hypot(points[0]![0] - points.at(-1)![0], points[0]![1] - points.at(-1)![1]) < 1e-5) {
    points.pop();
  }
  return points.length >= 3 && points.every((p) => p.every((v) => Number.isFinite(v) && Math.abs(v) <= 32768))
    ? points
    : null;
}
