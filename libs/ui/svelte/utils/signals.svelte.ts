import { SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";

let _isThisWebviewFocused = $state(false);
subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: { hwnd, ownerHwnd } }) => {
  // Guard against null/0 ids: when focus goes to an owner-less window, ownerHwnd
  // can be 0/null and match a not-yet-initialized self id, which briefly marks the
  // widget as focused and flashes auto-hidden docks/toolbars. Only match real ids.
  const self = Widget.self.windowId;
  _isThisWebviewFocused = (!!self && self === hwnd) || (!!ownerHwnd && self === ownerHwnd);
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
