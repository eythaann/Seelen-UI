/// <reference lib="dom" />

import assert from "node:assert/strict";
import { test } from "node:test";
import { handleGridKeyDown, navigateInDirection } from "./keyboard-navigation.ts";

test("Apps Menu Enter launches only the selected item in a folder", (t) => {
  class DialogTarget {
    matches(selector: string) {
      return selector.includes("dialog");
    }
  }
  Object.defineProperty(globalThis, "HTMLElement", { configurable: true, value: DialogTarget });
  t.after(() => {
    Reflect.deleteProperty(globalThis, "HTMLElement");
  });

  const launchFirst = t.mock.fn();
  const launchShortcut = t.mock.fn();
  const first = { dataset: { itemId: "first" }, click: launchFirst };
  const shortcut = { dataset: { itemId: String.raw`c:\start menu\Firefox.lnk` }, click: launchShortcut };
  let selected: string | null = null;
  const scope = {
    container: () => ({ querySelectorAll: () => [first, shortcut] }) as unknown as ParentNode,
    getSelected: () => selected,
    setSelected: (id: string | null) => {
      selected = id;
    },
  };
  const enter = {
    key: "Enter",
    target: new DialogTarget(),
    preventDefault() {},
  } as unknown as KeyboardEvent;

  handleGridKeyDown(enter, scope);
  scope.setSelected("stale");
  handleGridKeyDown(enter, scope);
  assert.equal(launchFirst.mock.callCount(), 0);
  assert.equal(launchShortcut.mock.callCount(), 0);

  scope.setSelected(shortcut.dataset.itemId);
  handleGridKeyDown(enter, scope);
  assert.equal(launchFirst.mock.callCount(), 0);
  assert.equal(launchShortcut.mock.callCount(), 1);
});

test("Apps Menu arrows move focus when navigating from a tabbed item", () => {
  const document = { activeElement: null as unknown };
  const first = {
    dataset: { itemId: "first" },
    ownerDocument: document,
    getBoundingClientRect: () => ({ left: 0, top: 0 }),
  };
  const second = {
    dataset: { itemId: "second" },
    ownerDocument: document,
    getBoundingClientRect: () => ({ left: 1, top: 0 }),
    focus: () => {
      document.activeElement = second;
    },
  };
  let selected: string | null = "first";
  const scope = {
    container: () => ({ querySelectorAll: () => [first, second] }) as unknown as ParentNode,
    getSelected: () => selected,
    setSelected: (id: string | null) => {
      selected = id;
    },
  };

  navigateInDirection("right", scope);
  assert.equal(selected, "second");
  assert.equal(document.activeElement, null);

  document.activeElement = first;
  navigateInDirection("right", scope);
  assert.equal(selected, "second");
  assert.equal(document.activeElement, second);
});
