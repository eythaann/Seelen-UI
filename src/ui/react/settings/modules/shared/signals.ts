import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { lazySignal } from "libs/ui/react/utils/LazySignal";

export const $virtual_desktops = lazySignal(async () => {
  return await invoke(SeelenCommand.StateGetVirtualDesktops);
});

await subscribe(SeelenEvent.VirtualDesktopsChanged, (event) => {
  $virtual_desktops.value = event.payload;
});

await $virtual_desktops.init();
