import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SysTrayIcon } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

const trayItems = lazyRune<SysTrayIcon[]>(() => invoke(SeelenCommand.GetSystemTrayIcons));
// listen first, so a change emitted while the icons are fetched is not lost
await subscribe(SeelenEvent.SystemTrayChanged, trayItems.setByPayload);
await trayItems.init();

class TrayState {
  get trayItems() {
    return trayItems.value;
  }
}

export const state = new TrayState();
