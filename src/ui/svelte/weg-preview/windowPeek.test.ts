import assert from "node:assert/strict";
import { test } from "node:test";
import { type PeekBridge, type PeekStatus, WindowPeekController } from "./windowPeek.ts";

const rect = { left: 10, top: 10, right: 200, bottom: 150 };
const active: PeekStatus = { active: true, committed: false, hwnd: 17 };

function fixture(): { controller: WindowPeekController; calls: unknown[][]; bridge: PeekBridge } {
  const calls: unknown[][] = [];
  let token = 0;
  const bridge: PeekBridge = {
    begin: (hwnd, session) => {
      calls.push(["begin", hwnd, session]);
      return Promise.resolve(active);
    },
    poll: (session, bounds) => {
      calls.push(["poll", session, bounds]);
      return Promise.resolve(active);
    },
    cancel: (session) => {
      calls.push(["cancel", session]);
      return Promise.resolve();
    },
    commit: (session) => {
      calls.push(["commit", session]);
      return Promise.resolve(true);
    },
    activate: (hwnd) => {
      calls.push(["activate", hwnd]);
      return Promise.resolve();
    },
  };
  return {
    controller: new WindowPeekController(bridge, () => calls.push(["hide"]), () => `session-${++token}`),
    calls,
    bridge,
  };
}

test("hover must dwell; leaving early never shows the real window", async () => {
  const { controller, calls } = fixture();
  controller.hover(17, 0);
  await controller.tick(rect, 449);
  controller.hover(null, 450);
  await controller.tick(null, 900);
  assert.deepEqual(calls, []);
});

test("pointer transfer keeps native lease alive, then native leave restores it", async () => {
  const { controller, bridge, calls } = fixture();
  controller.hover(17, 0);
  await controller.tick(rect, 450);
  controller.hover(null, 451);
  await controller.tick(null, 530);
  assert.equal(controller.holding, true);
  assert.deepEqual(calls[1], ["poll", "session-1", null]);
  bridge.poll = () => Promise.resolve({ active: false, committed: false, hwnd: 17 });
  await controller.tick(null, 610);
  assert.equal(controller.holding, false);
  assert.equal(calls.some(([name]) => name === "hide"), false);
});

test("thumbnail click fixes the existing HWND without a toggle or early hide", async () => {
  const { controller, calls } = fixture();
  controller.hover(17, 0);
  await controller.tick(rect, 450);
  assert.equal(await controller.commit(17), true);
  assert.deepEqual(calls, [["begin", 17, "session-1"], ["commit", "session-1"], ["hide"]]);
});

test("native click on the real window closes the thumbnail and keeps the source", async () => {
  const { controller, bridge, calls } = fixture();
  controller.hover(17, 0);
  await controller.tick(rect, 450);
  bridge.poll = () => Promise.resolve({ active: false, committed: true, hwnd: 17 });
  await controller.tick(rect, 530);
  assert.equal(controller.holding, false);
  assert.deepEqual(calls.at(-1), ["hide"]);
  await controller.tick(rect, 1200);
  assert.equal(calls.filter(([name]) => name === "begin").length, 1);
});

test("cancelled pending begin cannot resurrect an old hover", async () => {
  const { controller, bridge, calls } = fixture();
  let finish!: (status: PeekStatus) => void;
  bridge.begin = () =>
    new Promise((resolve) => {
      finish = resolve;
    });
  controller.hover(17, 0);
  const pending = controller.tick(rect, 450);
  controller.cancel();
  finish(active);
  await pending;
  assert.equal(controller.holding, false);
  assert.deepEqual(calls, [["cancel", "session-1"]]);
});

test("switching thumbnails cancels the exact old session before the next dwell", async () => {
  const { controller, calls } = fixture();
  controller.hover(17, 0);
  await controller.tick(rect, 450);
  controller.hover(29, 451);
  await controller.tick(rect, 800);
  assert.deepEqual(calls.at(-1), ["cancel", "session-1"]);
  await controller.tick(rect, 901);
  assert.deepEqual(calls.at(-1), ["begin", 29, "session-2"]);
});

test("a click during pending begin waits for that same session", async () => {
  const { controller, bridge, calls } = fixture();
  let finish!: (status: PeekStatus) => void;
  bridge.begin = () =>
    new Promise((resolve) => {
      finish = resolve;
    });
  controller.hover(17, 0);
  const pending = controller.tick(rect, 450);
  const click = controller.commit(17);
  assert.equal(await controller.commit(17), false);
  assert.deepEqual(calls, []);
  finish(active);
  await pending;
  assert.equal(await click, true);
  assert.deepEqual(calls, [["commit", "session-1"], ["hide"]]);
});

test("failed foreground request retains the preview and reports failure", async () => {
  const { controller, bridge, calls } = fixture();
  bridge.commit = () => Promise.resolve(false);
  controller.hover(17, 0);
  await controller.tick(rect, 450);
  assert.equal(await controller.commit(17), false);
  assert.equal(controller.holding, true);
  assert.equal(calls.some(([name]) => name === "hide" || name === "activate"), false);
});

test("click without dwell activates the chosen HWND directly", async () => {
  const { controller, calls } = fixture();
  assert.equal(await controller.commit(29), true);
  assert.deepEqual(calls, [["activate", 29], ["hide"]]);
});
