import assert from "node:assert/strict";
import { test } from "node:test";
import { getEndTaskTargets } from "./endTask.ts";

const item = { path: "C:\\Apps\\QQ\\QQ.exe", relaunch: null };
const icon = {
  sessionId: "qq",
  executablePath: "c:\\apps\\qq\\qq.exe",
  isVisible: true,
  windowHandle: "1_2_3",
  uid: 0,
  callbackMessage: 1024,
};

test("ends the tray process when the application has no open windows", () => {
  const unrelated = { ...icon, executablePath: "C:\\Other\\QQ.exe", windowHandle: "4_5_6" };
  assert.deepEqual(getEndTaskTargets(item, [], [unrelated, icon]), ["1_2_3"]);
  assert.deepEqual(getEndTaskTargets(item, [], [unrelated]), []);
});

test("targets each verified process once even when it owns several windows", () => {
  const windows = [{ hwnd: "4_2_3" }, { hwnd: "5_6_7" }, { hwnd: "8_6_7" }, { hwnd: "9_0_0" }];
  assert.deepEqual(getEndTaskTargets(item, windows, [icon]), ["1_2_3", "5_6_7"]);
});
