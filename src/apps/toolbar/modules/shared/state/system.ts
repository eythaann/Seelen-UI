import { signal } from '@preact/signals';
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from '@seelen-ui/lib';

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

const initialDesktops = await invoke(SeelenCommand.StateGetVirtualDesktops);

export const $virtual_desktop = signal(initialDesktops.monitors[currentMonitorId]);

subscribe(SeelenEvent.VirtualDesktopsChanged, (e) => {
  $virtual_desktop.value = e.payload.monitors[currentMonitorId];
});