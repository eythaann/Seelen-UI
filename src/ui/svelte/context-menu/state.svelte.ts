import { Widget } from "@seelen-ui/lib";
import type { ContextMenu } from "@seelen-ui/lib/types";
import { emitTo } from "@tauri-apps/api/event";

const CLOSE_CONTEXT_MENU_CHAIN_EVENT = "close-context-menu-chain";

let data = $state<ContextMenu | null>(null);
let owner = $state<string | null>(null);
let forwardTo = $state<string | null>(null);

Widget.self.onTrigger(({ customArgs }) => {
  data = (customArgs?.menu as any) || null;
  owner = (customArgs?.owner as any) || null;
  forwardTo = (customArgs?.forwardTo as any) || null;
});

/** Closes this menu and, if this menu was opened as a submenu, cascades the close up to its parents. */
export function closeContextMenuChain(): void {
  Widget.self.hide();
  if (owner) {
    emitTo(owner, CLOSE_CONTEXT_MENU_CHAIN_EVENT);
  }
}
Widget.self.webview.listen(CLOSE_CONTEXT_MENU_CHAIN_EVENT, closeContextMenuChain);

class State {
  get data() {
    return data;
  }

  get owner() {
    return owner;
  }

  get forwardTo() {
    return forwardTo;
  }
}

export const state = new State();
