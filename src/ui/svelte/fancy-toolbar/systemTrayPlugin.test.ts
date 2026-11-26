import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { compileFunction } from "node:vm";
import type { SysTrayIcon } from "@seelen-ui/lib/types";

const renderTray = compileFunction(
  readFileSync(new URL("../../../static/widgets/system-tray/toolbar-plugin.js", import.meta.url), "utf8"),
  ["trayIcons", "self", "Button", "Image", "AppIcon", "Icon"],
);

const qq = {
  sessionId: "30528_3",
  isVisible: true,
  isPromoted: true,
  tooltip: "QQ",
  executablePath: "C:\\Program Files\\Tencent\\QQNT\\QQ.exe",
  iconPath: null,
  iconImageHash: null,
};

function renderIcon(icon: Pick<SysTrayIcon, keyof typeof qq>) {
  return renderTray(
    [icon],
    { placement: "right" },
    (props: unknown) => props,
    (props: unknown) => ({ kind: "Image", props }),
    (props: unknown) => ({ kind: "AppIcon", props }),
    (props: unknown) => ({ kind: "Icon", props }),
  )[0];
}

test("a promoted tray entry without an image uses its application icon and stays clickable", () => {
  const button = renderIcon(qq);
  assert.deepEqual(button.content, { kind: "AppIcon", props: { path: qq.executablePath } });
  assert.equal(button.tooltip, "QQ");
  assert.match(button.onClick, /30528_3/);
});

test("a promoted tray entry with a native image uses that image and its version", () => {
  const icon = { ...qq, iconPath: "C:\\Temp\\30528_3.png", iconImageHash: "new-version" };
  assert.deepEqual(renderIcon(icon).content, {
    kind: "Image",
    props: { path: icon.iconPath, version: icon.iconImageHash },
  });
});
