import { signal } from '@preact/signals';
import { WegItems, WegItemType, Widget } from '@seelen-ui/lib';
import { WegItem } from '@seelen-ui/lib/types';
import { debounce } from 'lodash';

import { SeparatorWegItem } from '../store/domain';

interface DockState {
  isReorderDisabled: boolean;
  items: WegItem[];
}

export const HardcodedSeparator1: SeparatorWegItem = {
  id: 'hardcoded-separator-1',
  type: WegItemType.Separator,
};

export const HardcodedSeparator2: SeparatorWegItem = {
  id: 'hardcoded-separator-2',
  type: WegItemType.Separator,
};

function getStateFromStored(raw: WegItems): DockState {
  return {
    isReorderDisabled: raw.inner.isReorderDisabled,
    items: [
      ...raw.inner.left,
      HardcodedSeparator1,
      ...raw.inner.center,
      HardcodedSeparator2,
      ...raw.inner.right,
    ],
  };
}

function stateToStored(state: DockState): WegItems {
  const index1 = state.items.indexOf(HardcodedSeparator1);
  const index2 = state.items.indexOf(HardcodedSeparator2);
  const filter = (item: WegItem) => item.type !== WegItemType.Temporal;

  return new WegItems({
    isReorderDisabled: state.isReorderDisabled,
    left: state.items.slice(0, index1).filter(filter),
    center: state.items.slice(index1 + 1, index2).filter(filter),
    right: state.items.slice(index2 + 1).filter(filter),
  });
}

let monitorId = Widget.getCurrent().decoded.monitorId!;
export const $dock_state = signal(getStateFromStored(await WegItems.getForMonitor(monitorId)));
WegItems.onChange(async () => {
  $dock_state.value = getStateFromStored(await WegItems.getForMonitor(monitorId));
});

$dock_state.subscribe(
  debounce((v) => {
    stateToStored(v).save();
  }, 1000),
);

export const $dock_state_actions = {
  remove(idToRemove: string) {
    $dock_state.value = {
      ...$dock_state.value,
      items: $dock_state.value.items.filter((item) => item.id !== idToRemove),
    };
  },
  pinApp(id: string) {
    $dock_state.value = {
      ...$dock_state.value,
      items: $dock_state.value.items.map((item) => {
        if (item.id === id && item.type === WegItemType.Temporal) {
          return { ...item, type: WegItemType.Pinned };
        }
        return item;
      }),
    };
  },
  unpinApp(id: string) {
    $dock_state.value = {
      ...$dock_state.value,
      items: $dock_state.value.items.map((item) => {
        if (item.id === id && item.type === WegItemType.Pinned) {
          return { ...item, type: WegItemType.Temporal };
        }
        return item;
      }),
    };
  },
  addMediaModule() {
    if (!$dock_state.value.items.some((current) => current.type === WegItemType.Media)) {
      const newItems = [...$dock_state.value.items];
      newItems.push({
        id: crypto.randomUUID(),
        type: WegItemType.Media,
      });
      $dock_state.value = { ...$dock_state.value, items: newItems };
    }
  },
  addStartModule() {
    if (!$dock_state.value.items.some((current) => current.type === WegItemType.StartMenu)) {
      const newItems = [...$dock_state.value.items];
      newItems.unshift({
        id: crypto.randomUUID(),
        type: WegItemType.StartMenu,
      });
      $dock_state.value = { ...$dock_state.value, items: newItems };
    }
  },
};
