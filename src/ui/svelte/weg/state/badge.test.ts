import assert from "node:assert/strict";
import { test } from "node:test";
import { BadgeGlyph } from "@seelen-ui/lib/types";
import { getBadge } from "./badge.ts";

const windowWithoutBadge = { badgeValue: null, badgeIconPath: null, badgeUpdatedAt: 0 };

test("leaves the Dock badge empty when the app provides no badge or clears its count", () => {
  assert.equal(getBadge([windowWithoutBadge]), null);
  assert.equal(getBadge([{ ...windowWithoutBadge, badgeValue: 0 }]), null);
});

test("uses the app's badge once for grouped windows", () => {
  const windowWithBadge = { ...windowWithoutBadge, badgeValue: 7 };
  assert.deepEqual(getBadge([windowWithBadge, windowWithBadge]), { type: "count", count: 7 });
});

test("preserves app-provided status glyphs", () => {
  assert.deepEqual(
    getBadge([{ ...windowWithoutBadge, badgeValue: BadgeGlyph.NewMessage }]),
    { type: "icon", name: "IoMail" },
  );
});
