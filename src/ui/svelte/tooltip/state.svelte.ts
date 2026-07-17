import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { ZOrder } from "@seelen-ui/lib/types";

let tooltipText = $state<string | null>(null);
const showing = $derived(!!tooltipText);

Widget.self.onTrigger(async ({ desiredPosition, alignX, alignY, customArgs }) => {
  const show = Boolean(customArgs?.show);
  if (!show) {
    tooltipText = null;
    return;
  }

  if (desiredPosition) {
    await Widget.self.adjustAndSetPosition(desiredPosition.x, desiredPosition.y, alignX, alignY);
    invoke(SeelenCommand.SetSelfZOrder, { zOrder: ZOrder.Top });
  }

  tooltipText = String(customArgs?.text ?? "") || null;
});

// The widget that triggered the tooltip (dock/toolbar) can lose the mouse
// without ever emitting a hide (e.g. `pointerleave` missed).
// As a safety net, close the tooltip whenever the focused app/window changes —
// that always means the user has moved on, regardless of which window (if any)
// held OS focus before.
let focusedAppHwnd = $state<number | null>(null);
subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: focused }) => {
  focusedAppHwnd = focused.hwnd;
});

$effect.root(() => {
  let initialized = false;
  $effect(() => {
    focusedAppHwnd;
    if (!initialized) {
      initialized = true;
      return;
    }
    tooltipText = null;
  });

  $effect(() => {
    if (showing) {
      Widget.self.show();
    } else {
      Widget.self.hide();
    }
  });
});

class State {
  get text() {
    return tooltipText;
  }

  get showing() {
    return showing;
  }
}

export const state = new State();
