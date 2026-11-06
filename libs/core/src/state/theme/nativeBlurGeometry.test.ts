/// <reference lib="deno.ns" />

import { assertEquals } from "@std/assert";
import { normalizeRadii, resolveRadius } from "./nativeBlurGeometry.ts";

Deno.test("native blur resolves elliptical percentage corners against the border box", () => {
  assertEquals(resolveRadius("50% 25%", 200, 80), [100, 20]);
  assertEquals(resolveRadius("12px", 200, 80), [12, 12]);
});

Deno.test("native blur proportionally normalizes overlapping CSS corners", () => {
  assertEquals(normalizeRadii([[80, 40], [80, 40], [80, 40], [80, 40]], 100, 50), [[50, 25], [50, 25], [50, 25], [
    50,
    25,
  ]]);
  assertEquals(normalizeRadii([[0, 0], [0, 0], [0, 0], [0, 0]], 20, 20), [[0, 0], [0, 0], [0, 0], [0, 0]]);
});
