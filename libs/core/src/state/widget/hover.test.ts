/// <reference lib="deno.ns" />

import { assertEquals } from "@std/assert";
import { FakeTime } from "@std/testing/time";
import { HoverDismissal } from "./hover.ts";

const popup = { left: 100, top: 100, right: 200, bottom: 200 };
const anchor = { left: 0, top: 0, right: 80, bottom: 80 };
const inside = { x: 50, y: 50 };
const outside = { x: 500, y: 500 };

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((finish) => resolve = finish);
  return { promise, resolve };
}

async function flush(): Promise<void> {
  for (let i = 0; i < 6; i++) await Promise.resolve();
}

class Pointer {
  listeners: ((x: number, y: number) => void)[] = [];
  ready = Promise.resolve();
  point = Promise.resolve(inside);
  reads = 0;
  unsubscribed = 0;

  subscribe(onMove: (x: number, y: number) => void): Promise<() => void> {
    this.listeners.push(onMove);
    return this.ready.then(() => () => this.unsubscribed++);
  }

  position(): Promise<{ x: number; y: number }> {
    this.reads++;
    return this.point;
  }

  move(point: { x: number; y: number }): void {
    this.listeners.at(-1)!(point.x, point.y);
  }
}

async function check(
  run: (test: { time: FakeTime; pointer: Pointer; hover: HoverDismissal; hidden: () => number }) => Promise<void>,
): Promise<void> {
  const time = new FakeTime();
  const pointer = new Pointer();
  let hidden = 0;
  const hover = new HoverDismissal(() => popup, () => {
    hidden++;
    return Promise.resolve();
  }, pointer);
  try {
    await run({ time, pointer, hover, hidden: () => hidden });
  } finally {
    hover.stop();
    await flush();
    time.restore();
  }
}

Deno.test("continuous outside motion does not postpone the first dismissal deadline", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    hover.start(anchor);
    await flush();
    pointer.move(outside);
    time.tick(100);
    pointer.move(outside);
    time.tick(49);
    assertEquals(hidden(), 0);
    time.tick(1);
    assertEquals(hidden(), 1);
  }));

Deno.test("returning through the anchor bridge cancels dismissal and leaving again starts a new grace period", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    hover.start(anchor);
    await flush();
    pointer.move(outside);
    time.tick(100);
    pointer.move(inside);
    time.tick(100);
    assertEquals(hidden(), 0);
    pointer.move(outside);
    time.tick(149);
    assertEquals(hidden(), 0);
    time.tick(1);
    assertEquals(hidden(), 1);
  }));

Deno.test("a slow-opening popup samples the stationary cursor after its listener becomes ready", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    const ready = deferred<void>();
    pointer.ready = ready.promise;
    pointer.point = Promise.resolve(outside);
    hover.start(anchor);
    await flush();
    assertEquals(pointer.reads, 0);
    ready.resolve();
    await flush();
    assertEquals(pointer.reads, 1);
    time.tick(150);
    assertEquals(hidden(), 1);
  }));

Deno.test("a delayed outside sample cannot override a newer move back inside", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    const sample = deferred<{ x: number; y: number }>();
    pointer.point = sample.promise;
    hover.start(anchor);
    await flush();
    pointer.move(inside);
    sample.resolve(outside);
    await flush();
    time.tick(150);
    assertEquals(hidden(), 0);
  }));

Deno.test("a delayed inside sample cannot cancel a newer outside dismissal deadline", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    const sample = deferred<{ x: number; y: number }>();
    pointer.point = sample.promise;
    hover.start(anchor);
    await flush();
    pointer.move(outside);
    time.tick(100);
    sample.resolve(inside);
    await flush();
    time.tick(50);
    assertEquals(hidden(), 1);
  }));

Deno.test("a restarted popup ignores old listeners and samples and checks its current cursor again", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    const oldSample = deferred<{ x: number; y: number }>();
    pointer.point = oldSample.promise;
    hover.start(anchor);
    await flush();
    pointer.point = Promise.resolve(inside);
    hover.start(anchor);
    await flush();
    oldSample.resolve(outside);
    pointer.listeners[0]!(outside.x, outside.y);
    await flush();
    time.tick(150);
    assertEquals(hidden(), 0);
    assertEquals(pointer.reads, 2);
    assertEquals(pointer.unsubscribed, 1);
    pointer.point = Promise.resolve(outside);
    hover.start(anchor);
    await flush();
    time.tick(150);
    assertEquals(hidden(), 1);
  }));

Deno.test("stopping while subscription is pending releases it without sampling or hiding", () =>
  check(async ({ time, pointer, hover, hidden }) => {
    const ready = deferred<void>();
    pointer.ready = ready.promise;
    hover.start(anchor);
    hover.stop();
    pointer.move(outside);
    ready.resolve();
    await flush();
    time.tick(150);
    assertEquals(pointer.reads, 0);
    assertEquals(pointer.unsubscribed, 1);
    assertEquals(hidden(), 0);
  }));
