import { SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";

let _isThisWebviewFocused = $state(false);
subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: { id, ownerId } }) => {
  _isThisWebviewFocused = Widget.self.windowId === id || Widget.self.windowId === ownerId;
});

export const isThisWebviewFocused = {
  get value() {
    return _isThisWebviewFocused;
  },
};

const _pointerQuery = window.matchMedia("(hover: hover) and (pointer: fine)");
let _isTouchPrimary = $state(!_pointerQuery.matches);
_pointerQuery.addEventListener("change", (e) => {
  _isTouchPrimary = !e.matches;
});

export const isTouchPrimary = {
  get value() {
    return _isTouchPrimary;
  },
};
