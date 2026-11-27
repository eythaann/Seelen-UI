import assert from "node:assert/strict";
import { test } from "node:test";
import type { MediaDeviceSession } from "@seelen-ui/lib/types";
import { groupSessionsByApp } from "./mixer.ts";

function session(overrides: Partial<MediaDeviceSession>): MediaDeviceSession {
  return {
    id: "id",
    instanceId: "instance",
    appId: "app",
    processId: 1,
    name: "App",
    iconPath: null,
    isSystem: false,
    volume: 1,
    muted: false,
    ...overrides,
  };
}

test("sessions of a same app become one row, even with different session ids", () => {
  const apps = groupSessionsByApp([
    session({ id: "a", instanceId: "a1", appId: "exe:discord.exe", name: "Discord", volume: 0.3 }),
    session({ id: "b", instanceId: "b1", appId: "exe:discord.exe", name: "Discord", volume: 0.8 }),
    session({ id: "c", instanceId: "c1", appId: "exe:spotify.exe", name: "Spotify" }),
  ]);
  assert.deepEqual(apps.map((app) => app.appId), ["exe:discord.exe", "exe:spotify.exe"]);
  assert.equal(apps[0]!.sessionId, "a1");
  assert.equal(apps[0]!.volume, 0.8);
});

test("an app is muted only when all its sessions are", () => {
  const partly = groupSessionsByApp([
    session({ instanceId: "1", muted: true }),
    session({ instanceId: "2", muted: false }),
  ]);
  assert.equal(partly[0]!.muted, false);

  const fully = groupSessionsByApp([
    session({ instanceId: "1", muted: true }),
    session({ instanceId: "2", muted: true }),
  ]);
  assert.equal(fully[0]!.muted, true);
});

test("rows are sorted by name", () => {
  const apps = groupSessionsByApp([
    session({ appId: "z", name: "Zoom" }),
    session({ appId: "b", name: "Brave" }),
  ]);
  assert.deepEqual(apps.map((app) => app.name), ["Brave", "Zoom"]);
});
