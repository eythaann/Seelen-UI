/// <reference lib="deno.ns" />
import { assert, assertEquals } from "@std/assert";
import { projectFlipOutline } from "./nativeBlurProjection.ts";

const radii: [[number, number], [number, number], [number, number], [number, number]] = [[18, 18], [18, 18], [18, 18], [
  18,
  18,
]];
function matrix(angle: number): number[] {
  const c = Math.cos(angle), s = Math.sin(angle);
  return [1, 0, 0, 0, 0, c, s, -s / 480, 0, -s, c, -c / 480, 0, 0, 0, 1];
}
Deno.test("flip projection keeps rounded convex silhouette within the browser bounds", () => {
  for (
    const [angle, origin] of [[-1.3, [200, 0]], [-.6, [100, 0]], [.6, [200, 600]], [1.3, [100, 600]]] as [
      number,
      [number, number],
    ][]
  ) {
    const bounds = { x: 137, y: 53, width: 430, height: 280 };
    const points = projectFlipOutline(400, 600, radii, matrix(angle), origin, bounds);
    assert(points && points.length <= 64);
    let area = 0;
    for (let i = 0; i < points.length; i++) {
      const p = points[i]!, q = points[(i + 1) % points.length]!, r = points[(i + 2) % points.length]!;
      assert(p[0] >= bounds.x - 1e-6 && p[0] <= bounds.x + bounds.width + 1e-6);
      assert(p[1] >= bounds.y - 1e-6 && p[1] <= bounds.y + bounds.height + 1e-6);
      assert((q[0] - p[0]) * (r[1] - q[1]) - (q[1] - p[1]) * (r[0] - q[0]) >= -1e-6);
      area += p[0] * q[1] - q[0] * p[1];
    }
    assert(area > 0);
  }
});
Deno.test("flip projection rejects back faces, shear and perspective crossing the camera", () => {
  const bounds = { x: 0, y: 0, width: 400, height: 600 };
  assertEquals(projectFlipOutline(400, 600, radii, matrix(Math.PI), [200, 0], bounds), null);
  const shear = matrix(.4);
  shear[4] = .1;
  assertEquals(projectFlipOutline(400, 600, radii, shear, [200, 0], bounds), null);
  const crossing = matrix(.4);
  crossing[7] = -.02;
  assertEquals(projectFlipOutline(400, 600, radii, crossing, [200, 0], bounds), null);
});
Deno.test("flip projection supports sharp corners and identity without duplicate vertices", () => {
  const points = projectFlipOutline(400, 600, [[0, 0], [0, 0], [0, 0], [0, 0]], matrix(0), [200, 0], {
    x: 10,
    y: 20,
    width: 400,
    height: 600,
  });
  assertEquals(points, [[10, 20], [410, 20], [410, 620], [10, 620]]);
});
