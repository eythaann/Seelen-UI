import { HideMode } from "@seelen-ui/lib/types";
import { SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { settingsState } from "./settings.svelte.ts";
import { mouseAtEdge } from "./system.svelte.ts";
import { isTbOverlapped } from "./windows.svelte.ts";

let _hiddenByAutohide = $state(false);

let _isWebviewFocused = $state(false);
subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: { hwnd, ownerHwnd } }) => {
  _isWebviewFocused = Widget.self.windowId === hwnd || Widget.self.windowId === ownerHwnd;
});

const _pointerQuery = window.matchMedia("(hover: hover) and (pointer: fine)");
let _isTouchPrimary = $state(!_pointerQuery.matches);
_pointerQuery.addEventListener("change", (e) => {
  _isTouchPrimary = !e.matches;
});

export const hiddenByAutohide = {
  get value() {
    return _hiddenByAutohide;
  },
};

$effect.root(() => {
  $effect(() => {
    const { delayToHide, delayToShow, hideMode, position } = settingsState;
    const isMouseOverEdge = mouseAtEdge.value === position;

    let hidden = false;
    let flush = false;

    switch (hideMode) {
      case HideMode.Never:
        hidden = false;
        flush = true;
        break;
      case HideMode.Always:
        hidden = !_isTouchPrimary && !_isWebviewFocused && !isMouseOverEdge;
        flush = _isTouchPrimary;
        break;
      case HideMode.OnOverlap:
        hidden = !_isTouchPrimary && isTbOverlapped.value && !_isWebviewFocused && !isMouseOverEdge;
        flush = _isTouchPrimary;
        break;
    }

    let timeout: ReturnType<typeof setTimeout> | null = null;
    if (hidden) {
      timeout = setTimeout(() => {
        _hiddenByAutohide = true;
      }, delayToHide);
    } else {
      if (flush) {
        _hiddenByAutohide = false;
      } else {
        timeout = setTimeout(() => {
          _hiddenByAutohide = false;
        }, delayToShow);
      }
    }

    return () => {
      if (timeout) clearTimeout(timeout);
    };
  });
});
