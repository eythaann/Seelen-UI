import { computed, signal } from '@preact/signals';
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from '@seelen-ui/lib';
import { SeelenWegSide } from '@seelen-ui/lib/types';

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

const $monitors = signal(await invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, (e) => {
  $monitors.value = e.payload;
});

const $current_monitor = computed(() => $monitors.value.find((m) => m.id === currentMonitorId)!);

const $mouse_pos = signal({ x: 0, y: 0 });
subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
  $mouse_pos.value = { x, y };
});

export const $mouse_at_edge = computed<SeelenWegSide | null>(() => {
  if ($mouse_pos.value.y === $current_monitor.value.rect.top) {
    return 'Top';
  }
  if ($mouse_pos.value.x === $current_monitor.value.rect.left) {
    return 'Left';
  }
  if ($mouse_pos.value.y === $current_monitor.value.rect.bottom - 1) {
    return 'Bottom';
  }
  if ($mouse_pos.value.x === $current_monitor.value.rect.right - 1) {
    return 'Right';
  }
  return null;
});
