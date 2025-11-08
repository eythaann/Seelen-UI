import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

export const state = $state({
  trayItems: await invoke(SeelenCommand.GetSystemTrayIcons),
});

subscribe(SeelenEvent.SystemTrayChanged, (e) => {
  state.trayItems = e.payload;
});
