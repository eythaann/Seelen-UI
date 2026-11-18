import { assertEquals } from "@std/assert";
import { NativeBlurLane } from "./nativeBlurLane.ts";

const tick = async (): Promise<void> => {
  for (let i = 0; i < 8; i++) await Promise.resolve();
};
function deferred(): { promise: Promise<void>; resolve: () => void; reject: (reason: unknown) => void } {
  let resolve!: () => void, reject!: (reason: unknown) => void;
  const promise = new Promise<void>((a, b) => {
    resolve = a;
    reject = b;
  });
  return { promise, resolve, reject };
}

Deno.test("one never-returning old epoch leaves a reusable current slot across many reopens", async () => {
  const old = deferred(), calls: number[] = [];
  let active = 0, maximum = 0;
  const lane = new NativeBlurLane<number>(async (value) => {
    calls.push(value);
    active++;
    maximum = Math.max(maximum, active);
    if (value === 1) await old.promise;
    active--;
  }, () => {
    throw new Error("unexpected failure");
  });
  lane.advance(1);
  lane.offer(1, 1);
  await tick();
  for (let epoch = 2; epoch < 100; epoch++) {
    lane.advance(epoch);
    lane.offer(epoch, epoch);
    await tick();
  }
  assertEquals(calls.length, 99);
  assertEquals(maximum, 2);
  old.resolve();
  await tick();
  assertEquals(active, 0);
});

Deno.test("same-epoch updates coalesce to the latest payload without extra concurrency", async () => {
  const first = deferred(), calls: number[] = [];
  const lane = new NativeBlurLane<number>(async (value) => {
    calls.push(value);
    if (value === 1) await first.promise;
  }, () => {});
  lane.advance(1);
  lane.offer(1, 1);
  await tick();
  for (let i = 2; i <= 100; i++) lane.offer(1, i);
  assertEquals(calls, [1]);
  first.resolve();
  await tick();
  assertEquals(calls, [1, 100]);
});

Deno.test("old rejection/finally cannot fail or clear a newer active operation", async () => {
  const old = deferred(), current = deferred(), calls: number[] = [], errors: unknown[] = [];
  const lane = new NativeBlurLane<number>(async (value) => {
    calls.push(value);
    if (value === 1) await old.promise;
    if (value === 2) await current.promise;
  }, (error) => errors.push(error));
  lane.advance(1);
  lane.offer(1, 1);
  await tick();
  lane.advance(2);
  lane.offer(2, 2);
  await tick();
  old.reject(new Error("late"));
  await tick();
  lane.offer(2, 3);
  await tick();
  assertEquals(errors, []);
  assertEquals(calls, [1, 2]);
  current.resolve();
  await tick();
  assertEquals(calls, [1, 2, 3]);
});

Deno.test("two stuck requests enforce a hard cap; recovery delivers only the latest epoch", async () => {
  const a = deferred(), b = deferred(), calls: number[] = [];
  const lane = new NativeBlurLane<number>(async (value) => {
    calls.push(value);
    if (value === 1) await a.promise;
    if (value === 2) await b.promise;
  }, () => {});
  lane.advance(1);
  lane.offer(1, 1);
  await tick();
  lane.advance(2);
  lane.offer(2, 2);
  await tick();
  for (let epoch = 3; epoch <= 100; epoch++) {
    lane.advance(epoch);
    lane.offer(epoch, epoch);
  }
  await tick();
  assertEquals(calls, [1, 2]);
  a.resolve();
  await tick();
  assertEquals(calls, [1, 2, 100]);
  b.resolve();
  await tick();
});

Deno.test("clear and epoch advance discard queued work; current failures are observed once", async () => {
  const gate = deferred(), calls: number[] = [], errors: unknown[] = [];
  const lane = new NativeBlurLane<number>(async (value) => {
    calls.push(value);
    if (value === 1) await gate.promise;
    if (value === 4) throw new Error("current");
  }, (error) => errors.push(error));
  lane.advance(1);
  lane.offer(1, 1);
  await tick();
  lane.offer(1, 2);
  lane.clear();
  gate.resolve();
  await tick();
  lane.advance(3);
  lane.offer(2, 3);
  await tick();
  assertEquals(calls, [1]);
  lane.offer(3, 4);
  await tick();
  assertEquals(errors.length, 1);
});
