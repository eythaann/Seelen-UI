import { HideMode } from "@seelen-ui/lib/types";
import { isThisWebviewFocused, isTouchPrimary } from "libs/ui/svelte/utils/signals.svelte.ts";
import { settingsState } from "./settings.svelte.ts";
import { systemState } from "./system.svelte.ts";
import { windowsState } from "./windows.svelte.ts";

let _hiddenByAutohide = $state(false);

export const hiddenByAutohide = {
  get value() {
    return _hiddenByAutohide;
  },
};

$effect.root(() => {
  $effect(() => {
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
        hidden = !isTouchPrimary.value && windowsState.isTbOverlapped && !isThisWebviewFocused.value &&
          !isMouseOverEdge;
        flush = isTouchPrimary.value;
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
