import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { test } from "node:test";
import { executableName, isMediaShortcut, matchingSessions, mediaShortcutProfile } from "./media.ts";
import { insidePreviewBridge } from "./hover.ts";

test("prefer exact app identity, not the system's default session", () => {
  const music = { umid: "NetEaseMusic" };
  const browser = { umid: "msedge.exe" };
  assertEquals(matchingSessions({ umid: "neteasemusic", process: { path: "C:/cloudmusic.exe" } }, [browser, music]), [
    music,
  ]);
});

test("executable identity accepts paths and ignores case", () => {
  const browser = { umid: "C:\\Program Files\\Edge\\msedge.exe" };
  assertEquals(executableName(browser.umid), "msedge.exe");
  assertEquals(matchingSessions({ umid: null, process: { path: "D:/MSedge.EXE" } }, [browser]), [browser]);
});

test("unmatched app never inherits another app's media controls", () => {
  const unrelated = [{ umid: "msedge.exe" }, { umid: "cloudmusic.exe.backup" }];
  assertEquals(matchingSessions({ umid: null, process: { path: "C:/cloudmusic.exe" } }, unrelated), []);
  assertEquals(matchingSessions({ umid: null, process: { path: null } }, unrelated), []);
});

test("keep all matching sources for an explicit source choice", () => {
  const sessions = [{ umid: "C:/A/msedge.exe" }, { umid: "C:/B/msedge.exe" }];
  assertEquals(matchingSessions({ umid: null, process: { path: "D:/msedge.exe" } }, sessions), sessions);
});

test("shortcut profile lookup ignores case and prefers the canonical edited profile", () => {
  const profiles = { "CloudMusic.EXE": { enabled: false }, "cloudmusic.exe": { enabled: true } };
  assertEquals(mediaShortcutProfile(profiles, "CLOUDMUSIC.EXE"), { enabled: true });
  assertEquals(mediaShortcutProfile({ "QQMusic.exe": { enabled: true } }, "qqmusic.exe"), { enabled: true });
});

test("player shortcut grammar accepts the configured NetEase chords", () => {
  for (const chord of ["Ctrl+0", "Ctrl+-", "Ctrl+=", " ctrl + alt + F12 ", "Shift+Right"]) {
    assertEquals(isMediaShortcut(chord), true, chord);
  }
});

test("shortcut grammar rejects scripts, media broadcasts and malformed chords", () => {
  for (
    const chord of [
      "",
      "0",
      "Ctrl",
      "Ctrl+Ctrl+P",
      "Ctrl+Control+P",
      "Ctrl+P+Q",
      "Win+P",
      "Ctrl+F25",
      "Ctrl+F01",
      "Ctrl+F001",
      "Ctrl+MediaPlayPause",
      "Ctrl+{P}",
    ]
  ) {
    assertEquals(isMediaShortcut(chord), false, chord);
  }
});

test("pointer can cross the gap between popup and dock, including negative monitor coordinates", () => {
  const source = { left: -500, top: 900, right: -450, bottom: 950 };
  const popup = { left: -620, top: 650, right: -350, bottom: 890 };
  assertEquals(insidePreviewBridge([-470, 895], popup, source), true);
  assertEquals(insidePreviewBridge([-460, 925], popup, source), true);
  assertEquals(insidePreviewBridge([-400, 700], popup, source), true);
  assertEquals(insidePreviewBridge([-700, 895], popup, source), false);
  assertEquals(insidePreviewBridge([-470, 960], popup, source), false);
});
