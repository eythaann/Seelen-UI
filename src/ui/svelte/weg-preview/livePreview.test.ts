import assert from "node:assert/strict";
import { test } from "node:test";
import { setImmediate } from "node:timers/promises";
import {
  contentViewport,
  LivePreviewController,
  type LivePreviewFrame,
  type LivePreviewStatus,
  visiblePreviewTargets,
} from "./livePreview.ts";

const frame: LivePreviewFrame = {
  targets: [{ hwnd: "17", rect: { left: 10, top: 40, right: 266, bottom: 184 } }],
  opacity: 255,
};
const status: LivePreviewStatus = { live: ["17"], visible: true };

function recorder(calls: LivePreviewFrame[], result: () => Promise<LivePreviewStatus> = () => Promise.resolve(status)) {
  return {
    update: (_session: string, value: LivePreviewFrame) => {
      calls.push(value);
      return result();
    },
    clear: () => Promise.resolve(),
  };
}

test("partly scrolled thumbnails keep their full layout and carry an exact visible clip", () => {
  const rect = { left: 0, top: -50, right: 200, bottom: 50 };
  const [target] = visiblePreviewTargets([{ hwnd: "17", rect }], { left: 0, top: 0, right: 200, bottom: 100 });
  assert.deepEqual(target?.rect, rect);
  assert.deepEqual(target?.clip, { left: 0, top: 0, right: 200, bottom: 50 });
});

test("clips on the bottom and sides never expand a previously constrained surface", () => {
  const rect = { left: -10, top: 80, right: 210, bottom: 180 };
  const viewport = { left: 0, top: 0, right: 200, bottom: 100 };
  const [target] = visiblePreviewTargets([{ hwnd: "17", rect }], viewport);
  assert.deepEqual(target?.clip, { left: 0, top: 80, right: 200, bottom: 100 });
  const [constrained] = visiblePreviewTargets(
    [{ hwnd: "17", rect, clip: { left: 30, top: 85, right: 150, bottom: 95 } }],
    viewport,
  );
  assert.deepEqual(constrained?.clip, { left: 30, top: 85, right: 150, bottom: 95 });
});

test("the clip viewport excludes transformed borders and scrollbars", () => {
  const viewport = contentViewport({ left: 10, top: 20, right: 210, bottom: 120 }, {
    offsetWidth: 400,
    offsetHeight: 200,
    clientLeft: 4,
    clientTop: 2,
    clientWidth: 380,
    clientHeight: 180,
  });
  assert.deepEqual(viewport, { left: 12, top: 21, right: 202, bottom: 111 });
});

test("large groups preserve all visible unique thumbnails for native selection and snapshot fallback", () => {
  const targets = Array.from({ length: 40 }, (_, index) => ({
    hwnd: String(index + 1),
    rect: { left: 0, top: index * 200, right: 256, bottom: index * 200 + 144 },
  }));
  const selected = visiblePreviewTargets(targets, { left: 0, top: 0, right: 256, bottom: 400 });
  assert.deepEqual(selected.map(({ hwnd }) => hwnd), ["1", "2"]);
  const crowded = targets.map((target) => ({ ...target, rect: frame.targets[0]!.rect }));
  const bounded = visiblePreviewTargets(crowded, { left: 0, top: 0, right: 300, bottom: 300 });
  assert.equal(bounded.length, 40);
});

test("scrolling registers later windows rather than always truncating the first sixteen", () => {
  const targets = Array.from({ length: 20 }, (_, index) => ({
    hwnd: String(index + 1),
    rect: { left: 0, top: index * 200 - 3200, right: 256, bottom: index * 200 - 3200 + 144 },
  }));
  assert.deepEqual(
    visiblePreviewTargets(targets, { left: 0, top: 0, right: 256, bottom: 400 }).map(({ hwnd }) => hwnd),
    ["17", "18"],
  );
});

test("visibility handles clipped cards, duplicates, empty viewports and edge contact", () => {
  const targets = [
    { hwnd: "1", rect: { left: -20, top: -20, right: 30, bottom: 30 } },
    { hwnd: "1", rect: { left: 20, top: 20, right: 40, bottom: 40 } },
    { hwnd: "2", rect: { left: 100, top: 0, right: 200, bottom: 50 } },
    { hwnd: "3", rect: { left: 0, top: 100, right: 50, bottom: 200 } },
    { hwnd: "4", rect: { left: 0, top: 0, right: 0, bottom: 0 } },
  ];
  assert.deepEqual(
    visiblePreviewTargets(targets, { left: 0, top: 0, right: 100, bottom: 100 }).map(({ hwnd }) => hwnd),
    ["1"],
  );
  assert.deepEqual(visiblePreviewTargets(targets, { left: 0, top: 0, right: 0, bottom: 0 }), []);
});

test("clip-only changes reach native composition", async () => {
  const calls: LivePreviewFrame[] = [];
  const controller = new LivePreviewController(recorder(calls), () => {}, () => "clipped-surface");
  for (const bottom of [100, 75, 50]) {
    controller.update({
      ...frame,
      targets: [{ ...frame.targets[0]!, clip: { left: 10, top: 40, right: 266, bottom } }],
    });
    await setImmediate();
  }
  assert.deepEqual(calls.map((value) => value.targets[0]!.clip!.bottom), [100, 75, 50]);
});

test("a seventeenth visible card reaches snapshot fallback when sixteen native surfaces succeed", async () => {
  const targets = visiblePreviewTargets(
    Array.from({ length: 17 }, (_, index) => ({ hwnd: String(index + 1), rect: frame.targets[0]!.rect })),
    { left: 0, top: 0, right: 300, bottom: 300 },
  );
  const fallback: string[] = [];
  const controller = new LivePreviewController({
    update: (_session, value) => Promise.resolve({ ...status, live: value.targets.slice(0, 16).map((t) => t.hwnd) }),
    clear: () => Promise.resolve(),
  }, (state, value) => {
    fallback.push(...value.targets.filter((target) => !state.live.includes(target.hwnd)).map((target) => target.hwnd));
  });
  controller.update({ ...frame, targets });
  await setImmediate();
  assert.deepEqual(fallback, ["17"]);
});

test("a stable layout is sent once: video frames never cross the webview boundary", async () => {
  const calls: LivePreviewFrame[] = [];
  const controller = new LivePreviewController(recorder(calls), () => {}, () => "one-surface");
  for (let i = 0; i < 60; i++) {
    controller.update({ ...frame });
    await setImmediate();
  }
  assert.equal(calls.length, 1);
});

test("a rejected layout is resent on the next update", async (t) => {
  t.mock.method(console, "error", () => {});
  const calls: LivePreviewFrame[] = [];
  let fail = true;
  const controller = new LivePreviewController(
    recorder(calls, () => (fail ? Promise.reject(new Error("source closed")) : Promise.resolve(status))),
    () => {},
    () => "retry-session",
  );
  controller.update(frame);
  await setImmediate();
  fail = false;
  controller.update(frame);
  await setImmediate();
  assert.equal(calls.length, 2);
});

test("slow IPC coalesces animation layouts into the newest frame with one request in flight", async () => {
  let finish!: (value: LivePreviewStatus) => void;
  const calls: LivePreviewFrame[] = [];
  const controller = new LivePreviewController(
    recorder(calls, () =>
      calls.length === 1
        ? new Promise((resolve) => {
          finish = resolve;
        })
        : Promise.resolve(status)),
    () => {},
    () => "one-surface",
  );
  controller.update({ ...frame, opacity: 0 });
  for (let opacity = 1; opacity <= 255; opacity++) controller.update({ ...frame, opacity });
  assert.deepEqual(calls.map((value) => value.opacity), [0]);
  finish(status);
  await setImmediate();
  assert.deepEqual(calls.map((value) => value.opacity), [0, 255]);
});

test("late state from a cleared surface cannot mark a different app live", async () => {
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
  controller.update(frame);
  controller.clear();
  finish(status);
  await setImmediate();
  assert.deepEqual(seen, []);
  assert.deepEqual(cleared, ["session-1"]);
});

test("a new trigger starts a fresh session after an old pending layout completes", async () => {
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
  controller.update(frame);
  controller.clear();
  controller.update(frame);
  finish(status);
  await setImmediate();
  assert.deepEqual(tokens, ["session-1", "session-2"]);
});

test("opacity changes are sent while geometry stays fixed", async () => {
  const calls: LivePreviewFrame[] = [];
  const controller = new LivePreviewController(recorder(calls), () => {}, () => "one-surface");
  for (const opacity of [0, 80, 160, 255]) {
    controller.update({ ...frame, opacity });
    await setImmediate();
  }
  assert.deepEqual(calls.map((value) => value.opacity), [0, 80, 160, 255]);
});
