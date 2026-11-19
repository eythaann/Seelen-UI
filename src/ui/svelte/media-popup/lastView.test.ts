import assert from "node:assert/strict";
import { test } from "node:test";
import { DEFAULT_VIEW, ensureDeviceAvailable, resolveLastView, serializeLastView } from "./lastView.ts";

const devices = ["speakers", "headset", "mic"];

test("restores the mixer of a device that is still available", () => {
  const stored = serializeLastView({ view: "mixer", deviceId: "headset" });
  assert.deepEqual(resolveLastView(stored, devices), { view: "mixer", deviceId: "headset" });
});

test("falls back to the overview when the remembered device is gone", () => {
  const stored = serializeLastView({ view: "mixer", deviceId: "unplugged-dac" });
  assert.deepEqual(resolveLastView(stored, devices), DEFAULT_VIEW);
});

test("restores the overview when it was the last view", () => {
  const stored = serializeLastView({ view: "main", deviceId: null });
  assert.deepEqual(resolveLastView(stored, devices), DEFAULT_VIEW);
});

test("ignores missing, malformed or unexpected stored values", () => {
  for (const stored of [null, "", "not json", "null", "42", '{"view":"mixer"}', '{"view":"other","deviceId":"mic"}']) {
    assert.deepEqual(resolveLastView(stored, devices), DEFAULT_VIEW, `stored: ${stored}`);
  }
});

test("rechecks the current view on reopen: keeps an available device, drops a missing one", () => {
  assert.deepEqual(ensureDeviceAvailable({ view: "mixer", deviceId: "mic" }, devices), {
    view: "mixer",
    deviceId: "mic",
  });
  assert.deepEqual(ensureDeviceAvailable({ view: "mixer", deviceId: "unplugged-dac" }, devices), DEFAULT_VIEW);
  assert.deepEqual(ensureDeviceAvailable({ view: "mixer", deviceId: null }, devices), DEFAULT_VIEW);
  assert.deepEqual(ensureDeviceAvailable(DEFAULT_VIEW, devices), DEFAULT_VIEW);
});

test("never serializes a mixer view without a device", () => {
  assert.deepEqual(JSON.parse(serializeLastView({ view: "mixer", deviceId: null })), DEFAULT_VIEW);
});
