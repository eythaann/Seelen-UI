import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { test } from "node:test";
import { MediaCommand } from "./MediaCommand.ts";

test("rapid repeated clicks send one command while the first is pending", async () => {
  let sent = 0;
  let complete!: () => void;
  const operation = new Promise<void>((resolve) => complete = resolve);
  const states: unknown[] = [];
  const controller = new MediaCommand((state) => states.push(state));
  controller.setTarget("music");
  const pending = controller.run(() => {
    sent++;
    return operation;
  });
  assertEquals(
    await controller.run(() => {
      sent++;
      return operation;
    }),
    false,
  );
  assertEquals(sent, 1);
  assertEquals(states.at(-1), { pending: true, failed: false });
  complete();
  assertEquals(await pending, true);
  assertEquals(states.at(-1), { pending: false, failed: false });
});

test("failed command displays feedback and permits an explicit retry", async () => {
  let state: unknown;
  const controller = new MediaCommand((value) => state = value);
  controller.setTarget("music");
  assertEquals(await controller.run(() => Promise.reject(new Error("Session closed"))), false);
  assertEquals(state, { pending: false, failed: true });
  assertEquals(await controller.run(() => Promise.resolve()), true);
  assertEquals(state, { pending: false, failed: false });
});

test("late reply from an old source cannot clear the new source's pending state", async () => {
  let state: unknown;
  let rejectOld!: (reason: unknown) => void;
  let finishNew!: () => void;
  const controller = new MediaCommand((value) => state = value);
  controller.setTarget("music");
  const old = controller.run(() => new Promise((_, reject) => rejectOld = reject));
  controller.setTarget("browser");
  const current = controller.run(() => new Promise<void>((resolve) => finishNew = resolve));
  rejectOld(new Error("Old session closed"));
  await old;
  assertEquals(state, { pending: true, failed: false });
  finishNew();
  await current;
  assertEquals(state, { pending: false, failed: false });
});

test("a hung command times out without automatically retrying or claiming playback changed", async () => {
  let state: unknown;
  let sent = 0;
  const controller = new MediaCommand((value) => state = value, 1);
  controller.setTarget("music");
  assertEquals(
    await controller.run(() => {
      sent++;
      return new Promise(() => {});
    }),
    false,
  );
  assertEquals(state, { pending: false, failed: true });
  assertEquals(sent, 1);
});
