import type { AppOrFileWegItem } from "../types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { notifications } from "./getters.svelte";
import { BadgeGlyph, type UserAppWindow } from "@seelen-ui/lib/types";
import type { IconName } from "libs/ui/icons";

function getGlyphIconName(glyph: BadgeGlyph) {
  switch (glyph) {
    case BadgeGlyph.Activity:
      return "TbRefresh";
    case BadgeGlyph.Alarm:
      return "IoAlarm";
    case BadgeGlyph.Alert:
    case BadgeGlyph.Attention:
      return "IoAlert";
    case BadgeGlyph.Available:
      return "var(--color-green-400)";
    case BadgeGlyph.Away:
      return "var(--color-orange-400)";
    case BadgeGlyph.Busy:
      return "var(--color-red-400)";
    case BadgeGlyph.Error:
      return "IoClose";
    case BadgeGlyph.NewMessage:
      return "IoMail";
    case BadgeGlyph.Paused:
      return "IoPause";
    case BadgeGlyph.Playing:
      return "IoPlay";
    case BadgeGlyph.Unavailable:
      return "IoBan";
    case BadgeGlyph.None:
    case BadgeGlyph.Unknown:
      return "";
  }
}

type Badge =
  | { type: "count"; count: number }
  | { type: "color"; color: string }
  | { type: "icon"; name: IconName }
  | { type: "image"; src: string };

export function getBadge(item: AppOrFileWegItem, windows: UserAppWindow[]): Badge | null {
  const badgeValue = windows.find((w) => w.badgeValue)?.badgeValue;
  if (badgeValue) {
    if (typeof badgeValue === "number") {
      return { type: "count", count: Math.min(badgeValue, 99) };
    }

    const iconName = getGlyphIconName(badgeValue);
    if (iconName) {
      if (iconName.startsWith("var")) {
        return { type: "color", color: iconName };
      }
      return { type: "icon", name: iconName as IconName };
    }
  }

  const winWithBadge = windows
    .toSorted((a, b) => b.badgeUpdatedAt - a.badgeUpdatedAt)
    .find((a) => a.badgeIconPath);
  if (winWithBadge && winWithBadge.badgeIconPath && winWithBadge.badgeUpdatedAt) {
    return {
      type: "image",
      src: convertFileSrc(winWithBadge.badgeIconPath) + `?v=${winWithBadge.badgeUpdatedAt}`,
    };
  }

  const notificationsCount = notifications.value.filter((n) => n.appUmid === item.umid).length;
  if (notificationsCount) {
    return { type: "count", count: Math.min(notificationsCount, 99) };
  }

  return null;
}
