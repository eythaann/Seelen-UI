import assert from "node:assert/strict";
import { test } from "node:test";
import { setImmediate } from "node:timers/promises";
import {
  LivePreviewController,
  type LivePreviewFrame,
  type LivePreviewStatus,
  visiblePreviewTargets,
} from "./livePreview.ts";

const frame: LivePreviewFrame = {
  targets: [{ hwnd: 17, rect: { left: 10, top: 40, right: 266, bottom: 184 } }],
  opacity: 255,
  anchor: { left: -500, top: 100, right: -450, bottom: 150 },
};
const status: LivePreviewStatus = { live: [17], visible: true, pointerInside: true };

test("large groups only register visible unique thumbnails within the native budget", () => {
  const targets = Array.from({ length: 40 }, (_, index) => ({
    hwnd: index + 1,
    rect: { left: 0, top: index * 200, right: 256, bottom: index * 200 + 144 },
  }));
  const selected = visiblePreviewTargets(targets, { left: 0, top: 0, right: 256, bottom: 400 });
  assert.deepEqual(selected.map(({ hwnd }) => hwnd), [1, 2]);
  const crowded = targets.map((target) => ({ ...target, rect: frame.targets[0]!.rect }));
  const bounded = visiblePreviewTargets(crowded, { left: 0, top: 0, right: 300, bottom: 300 }, 40);
  assert.equal(bounded.length, 16);
  assert.equal(bounded[0]!.hwnd, 40);
});

test("scrolling registers later windows rather than always truncating the first sixteen", () => {
  const targets = Array.from({ length: 20 }, (_, index) => ({
    hwnd: index + 1,
    rect: { left: 0, top: index * 200 - 3200, right: 256, bottom: index * 200 - 3200 + 144 },
  }));
  assert.deepEqual(
    visiblePreviewTargets(targets, { left: 0, top: 0, right: 256, bottom: 400 }).map(({ hwnd }) => hwnd),
    [17, 18],
  );
});

test("visibility handles clipped cards, duplicates, empty viewports and edge contact", () => {
  const targets = [
    { hwnd: 1, rect: { left: -20, top: -20, right: 30, bottom: 30 } },
    { hwnd: 1, rect: { left: 20, top: 20, right: 40, bottom: 40 } },
    { hwnd: 2, rect: { left: 100, top: 0, right: 200, bottom: 50 } },
    { hwnd: 3, rect: { left: 0, top: 100, right: 50, bottom: 200 } },
    { hwnd: 4, rect: { left: 0, top: 0, right: 0, bottom: 0 } },
  ];
  assert.deepEqual(
    visiblePreviewTargets(targets, { left: 0, top: 0, right: 100, bottom: 100 }).map(({ hwnd }) => hwnd),
    [1],
  );
  assert.deepEqual(visiblePreviewTargets(targets, { left: 0, top: 0, right: 0, bottom: 0 }), []);
});

test("rejected target updates still recover pointer state and back off before retrying", async (t) => {
  t.mock.method(console, "error", () => {});
  let requests = 0;
  const states: LivePreviewStatus[] = [];
  const controller = new LivePreviewController(
    {
      update: () => {
        requests++;
        return Promise.reject(new Error("source closed"));
      },
      recover: () => Promise.resolve({ live: [], visible: true, pointerInside: false }),
      clear: () => Promise.resolve(),
    },
    (value) => states.push(value),
    () => "recovery-session",
  );
  controller.update(frame, 0);
  await setImmediate();
  assert.deepEqual(states, [{ live: [], visible: true, pointerInside: false }]);
  for (const now of [16, 80, 160, 240]) controller.update(frame, now);
  assert.equal(requests, 1);
  controller.update(frame, 250);
  await setImmediate();
  assert.equal(requests, 2);
});

test("late pointer recovery from a cleared session cannot hide a new popup", async (t) => {
  t.mock.method(console, "error", () => {});
  let finish!: (value: LivePreviewStatus) => void;
  const states: LivePreviewStatus[] = [];
  const controller = new LivePreviewController(
    {
      update: () => Promise.reject(new Error("rejected frame")),
      recover: () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
      clear: () => Promise.resolve(),
    },
    (value) => states.push(value),
    () => "old-session",
  );
  controller.update(frame, 0);
  await setImmediate();
  controller.clear();
  finish({ live: [], visible: true, pointerInside: false });
  await setImmediate();
  assert.deepEqual(states, []);
});

test("stable video surfaces only send heartbeats, not per-frame snapshots or registrations", async () => {
  const calls: number[] = [];
  const controller = new LivePreviewController(
    {
      update: (_session, value) => {
        calls.push(value.opacity);
        return Promise.resolve(status);
      },
      clear: () => Promise.resolve(),
    },
    () => {},
    () => "one-surface",
  );
  for (let now = 0; now < 1000; now += 16) {
    controller.update(frame, now);
    await setImmediate();
  }
  assert.equal(calls.length, 13);
});

test("slow IPC coalesces animation layouts into the newest frame with one request in flight", async () => {
  let finish!: (value: LivePreviewStatus) => void;
  const calls: number[] = [];
  const controller = new LivePreviewController(
    {
      update: (_session, value) => {
        calls.push(value.opacity);
        return calls.length === 1
          ? new Promise((resolve) => {
            finish = resolve;
          })
          : Promise.resolve(status);
      },
      clear: () => Promise.resolve(),
    },
    () => {},
    () => "one-surface",
  );
  controller.update({ ...frame, opacity: 0 }, 0);
  for (let opacity = 1; opacity <= 255; opacity++) controller.update({ ...frame, opacity }, opacity);
  assert.deepEqual(calls, [0]);
  finish(status);
  await setImmediate();
  assert.deepEqual(calls, [0, 255]);
});

test("late state from a cancelled surface cannot mark a different app live", async () => {
  let finish!: (value: LivePreviewStatus) => void;
  const seen: LivePreviewStatus[] = [];
  const cleared: string[] = [];
  let count = 0;
  const controller = new LivePreviewController(
    {
      update: () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
      clear: (session) => {
        cleared.push(session);
        return Promise.resolve();
      },
    },
    (value) => seen.push(value),
    () => `session-${++count}`,
  );
  controller.update(frame, 0);
  controller.clear();
  finish(status);
  await setImmediate();
  assert.deepEqual(seen, []);
  assert.deepEqual(cleared, ["session-1"]);
});

test("a new trigger starts a fresh lease after an old pending layout completes", async () => {
  let finish!: (value: LivePreviewStatus) => void;
  const tokens: string[] = [];
  let count = 0;
  const controller = new LivePreviewController(
    {
      update: (session) => {
        tokens.push(session);
        return tokens.length === 1
          ? new Promise((resolve) => {
            finish = resolve;
          })
          : Promise.resolve(status);
      },
      clear: () => Promise.resolve(),
    },
    () => {},
    () => `session-${++count}`,
  );
  controller.update(frame, 0);
  controller.clear();
  controller.update(frame, 16);
  finish(status);
  await setImmediate();
  assert.deepEqual(tokens, ["session-1", "session-2"]);
});

test("opacity changes are sent on animation frames while geometry stays fixed", async () => {
  const opacities: number[] = [];
  const controller = new LivePreviewController(
    {
      update: (_session, value) => {
        opacities.push(value.opacity);
        return Promise.resolve(status);
      },
      clear: () => Promise.resolve(),
    },
    () => {},
    () => "one-surface",
  );
  for (const [now, opacity] of [[0, 0], [16, 80], [32, 160], [48, 255]]) {
    controller.update({ ...frame, opacity: opacity! }, now!);
    await setImmediate();
  }
  assert.deepEqual(opacities, [0, 80, 160, 255]);
});
