import { HideMode } from "@seelen-ui/lib/types";
import { virtualDesktops } from "./getters.svelte.ts";
import { settingsState } from "./settings.svelte.ts";
import { systemState } from "./system.svelte.ts";
import { windowsState } from "./windows.svelte.ts";
import { isThisWebviewFocused, isTouchPrimary } from "libs/ui/svelte/utils";

const isSwitchingWorkspace = $derived(virtualDesktops.value.switching);

let _hiddenByAutohide = $state(false);
let _isDraggingItem = $state(false);

export const dockShouldBeHidden = {
  get value() {
    return _hiddenByAutohide;
  },
};

export function setDockIsDraggingItem(isDragging: boolean): void {
  _isDraggingItem = isDragging;
}

$effect.root(() => {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (isSwitchingWorkspace) {
      if (timeout) {
        clearTimeout(timeout);
        timeout = null;
      }
      return;
    }

    const { delayToHide, delayToShow, hideMode, position } = settingsState;
    const isMouseOverEdge = systemState.mouseAtEdge === position;

    let hidden = false;
    let flush = false;

    switch (hideMode) {
      case HideMode.Never:
        hidden = false;
        flush = true;
        break;
      case HideMode.Always:
        hidden = !isTouchPrimary.value && !isThisWebviewFocused.value && !isMouseOverEdge;
        flush = isTouchPrimary.value;
        break;
      case HideMode.OnOverlap:
        hidden = !isTouchPrimary.value &&
          windowsState.isDockOverlapped &&
          !isThisWebviewFocused.value &&
          !isMouseOverEdge;
        flush = isTouchPrimary.value;
        break;
    }

    if (_isDraggingItem) {
      hidden = false;
      flush = true;
    }

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
      if (timeout) {
        clearTimeout(timeout);
        timeout = null;
      }
    };
  });
});
