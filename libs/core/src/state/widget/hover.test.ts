/// <reference lib="deno.ns" />

import { assertEquals } from "@std/assert";
import { HoverDismissal } from "./hover.ts";

const popup = { left: 10, top: 10, right: 100, bottom: 100 };

Deno.test("a hover popup closes when the pointer already left and stopped before it opened", async () => {
  let hides = 0;
  const dismiss = new HoverDismissal(() => popup, async () => {
    hides++;
  }, {
    position: () => Promise.resolve([200, 200]),
    listen: () => Promise.resolve(() => {}),
  });

  dismiss.start(null);
  await new Promise((resolve) => setTimeout(resolve, 200));
  dismiss.stop();

  assertEquals(hides, 1);
});
