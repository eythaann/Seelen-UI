import assert from "node:assert/strict";
import { test } from "node:test";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import type { AppOrFileWegItem } from "../types.ts";
import { AppWindowsIndex } from "./appWindowsIndex.ts";

function window(hwnd: string, umid: string | null, path: string): UserAppWindow {
  return { hwnd, umid, process: { path } } as UserAppWindow;
}

const item = {
  umid: "app-id",
  path: "C:\\Apps\\App.exe",
  relaunch: { command: "C:\\Apps\\Relaunch.exe" },
} as AppOrFileWegItem;

test("merges AUMID and both path matches in source order without duplicates", () => {
  const windows = [
    window("path-first", null, "c:\\apps\\app.exe"),
    window("umid", "app-id", "C:\\Other.exe"),
    window("relaunch", null, "c:\\apps\\relaunch.exe"),
    window("unrelated", "other", "C:\\Other.exe"),
  ];
  const index = new AppWindowsIndex(windows);
  const matches = index.get(item);
  assert.deepEqual(matches.map((match) => match.hwnd), ["path-first", "umid", "relaunch"]);
  assert.equal(index.get(item), matches);
  assert.deepEqual(index.get({ ...item, umid: "other" }).map((match) => match.hwnd), [
    "path-first",
    "relaunch",
    "unrelated",
  ]);
});

test("does not match empty paths", () => {
  const index = new AppWindowsIndex([window("empty", null, "")]);
  assert.deepEqual(index.get({ ...item, umid: null, path: "", relaunch: null }), []);
});
