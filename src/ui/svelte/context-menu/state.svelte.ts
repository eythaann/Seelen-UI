import { Widget } from "@seelen-ui/lib";
import type { ContextMenu } from "@seelen-ui/lib/types";
import { emitTo } from "@tauri-apps/api/event";

let data = $state<ContextMenu | null>(null);
let owner = $state<string | null>(null);
let forwardTo = $state<string | null>(null);

Widget.self.onTrigger(({ customArgs }) => {
  data = (customArgs?.menu as any) || null;
  owner = (customArgs?.owner as any) || null;
  forwardTo = (customArgs?.forwardTo as any) || null;
});

// Track the currently open submenu identifier (only relevant in the root/parent menu)
let _openSubmenuId: string | null = null;

/** Encode a decoded webview label to its base64url (no-pad) raw form */
function encodeWebviewLabel(decodedLabel: string): string {
  const bytes = new TextEncoder().encode(decodedLabel);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]!);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

/** Close the currently tracked submenu window, if any */
export async function closeOpenSubmenu(): Promise<void> {
  if (!_openSubmenuId) return;
  const label = encodeWebviewLabel(
    `@seelen/context-menu?instanceId=${_openSubmenuId}`,
  );
  _openSubmenuId = null;
  await emitTo(label, "contextmenu:close", {}).catch(() => {});
}

/** Register a new submenu as the currently open one */
export function setOpenSubmenu(id: string): void {
  _openSubmenuId = id;
}

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

  /** True when this context menu was opened by another context menu (i.e. it is a submenu) */
  get isSubmenu() {
    return owner !== null && forwardTo !== null && owner !== forwardTo;
  }
}

export const state = new State();
