import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { signal } from "@preact/signals";

export const uiColors = lazySignal(() => invoke(SeelenCommand.SystemGetColors));
await subscribe(SeelenEvent.ColorsChanged, uiColors.setByPayload);
await uiColors.init();

export const monitors = lazySignal(() => invoke(SeelenCommand.SystemGetMonitors));
await subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

export const autostart = signal(await invoke(SeelenCommand.GetAutoStartStatus));
export async function setAutoStart(enabled: boolean) {
  await invoke(SeelenCommand.SetAutoStart, { enabled });
  autostart.value = enabled;
}
