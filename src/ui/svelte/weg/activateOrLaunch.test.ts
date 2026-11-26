import assert from "node:assert/strict";
import { test } from "node:test";
import type { WindowIdentity } from "@seelen-ui/lib/types";
import { activateOrLaunch, getMatchingTrayIcons } from "./activateOrLaunch.ts";

const item = { path: "C:\\Apps\\QQ\\QQ.exe", relaunch: null };
const icon = {
  sessionId: "qq",
  executablePath: "c:\\apps\\qq\\qq.exe",
  isVisible: true,
  windowHandle: "1_2" as WindowIdentity,
  uid: 0,
  callbackMessage: 1024,
};

test("a pinned app remains detectable through its tray icons until they are removed", () => {
  const unrelated = { ...icon, executablePath: "C:\\Other\\QQ.exe" };
  assert.deepEqual(getMatchingTrayIcons(item, [unrelated, icon]), [icon]);
  assert.deepEqual(getMatchingTrayIcons(item, [unrelated]), []);
});

test("File Explorer launches a folder window instead of activating a Windows shell tray icon", async () => {
  const path = "C:\\Windows\\explorer.exe";
  const explorer = { path, relaunch: null };
  const shellIcon = { ...icon, executablePath: path };
  const calls: string[] = [];
  assert.deepEqual(getMatchingTrayIcons(explorer, [shellIcon]), []);
  await activateOrLaunch(explorer, () => Promise.resolve([shellIcon]), () => {
    calls.push("tray");
    return Promise.resolve();
  }, () => {
    calls.push("launch");
    return Promise.resolve();
  });
  assert.deepEqual(calls, ["launch"]);
});

for (
  const { name, icons, expected } of [
    { name: "restores the running app through its tray icon", icons: [icon], expected: ["qq"] },
    { name: "launches when the app has no tray icon", icons: [], expected: ["launch"] },
    {
      name: "ignores an executable with the same name in another directory",
      icons: [{ ...icon, executablePath: "C:\\Other\\QQ.exe" }],
      expected: ["launch"],
    },
    { name: "ignores hidden icons", icons: [{ ...icon, isVisible: false }], expected: ["launch"] },
    {
      name: "does not choose between multiple matching tray icons",
      icons: [icon, { ...icon, sessionId: "other-qq" }],
      expected: ["launch"],
    },
  ]
) {
  test(name, async () => {
    const calls: string[] = [];
    await activateOrLaunch(item, () => Promise.resolve(icons), (id) => {
      calls.push(id);
      return Promise.resolve();
    }, () => {
      calls.push("launch");
      return Promise.resolve();
    });
    assert.deepEqual(calls, expected);
  });
}
