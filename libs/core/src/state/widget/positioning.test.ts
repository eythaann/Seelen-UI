/// <reference lib="deno.ns" />

import { Alignment } from "@seelen-ui/types";
import { assertEquals } from "@std/assert";
import { adjustPositionByPlacement } from "./positioning.ts";

Deno.test("an aligned popup stays on its anchor monitor near a shared edge", () => {
  const monitors = [
    { rect: { left: 0, top: 0, right: 1920, bottom: 1080 }, isPrimary: true },
    { rect: { left: 1920, top: 0, right: 3840, bottom: 1080 }, isPrimary: false },
  ];
  const frame = adjustPositionByPlacement({
    frame: { x: 1940, y: 1000, width: 300, height: 200 },
    originX: Alignment.Center,
    originY: Alignment.End,
    monitorPoint: { x: 1940, y: 1020 },
  }, monitors);

  assertEquals(frame, { x: 1920, y: 800, width: 300, height: 200 });
});
