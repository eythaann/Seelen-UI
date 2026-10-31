import assert from "node:assert/strict";
import { test } from "node:test";
import { setImmediate } from "node:timers/promises";
import { LivePreviewController, type LivePreviewFrame, type LivePreviewStatus } from "./livePreview.ts";

const frame: LivePreviewFrame = {
  targets: [{ hwnd: 17, rect: { left: 10, top: 40, right: 266, bottom: 184 } }],
  opacity: 255,
  anchor: { left: -500, top: 100, right: -450, bottom: 150 },
};
const status: LivePreviewStatus = { live: [17], visible: true, pointerInside: true };

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
