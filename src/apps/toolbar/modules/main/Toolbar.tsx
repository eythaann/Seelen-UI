import {
  closestCorners,
  DndContext,
  DragEndEvent,
  DragOverEvent,
  DragOverlay,
  DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import { arrayMove } from '@dnd-kit/sortable';
import { useComputed, useSignal } from '@preact/signals';
import { ToolbarItem2 } from '@seelen-ui/lib/types';
import { isEqual } from 'lodash';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../shared/store/app';

import { RootState } from '../shared/store/domain';

import { AnimatedDropdown } from '../../../shared/components/AnimatedWrappers';
import { useWindowFocusChange } from '../../../shared/hooks';
import { cx } from '../../../shared/styles';
import { $toolbar_state } from '../shared/state/items';
import { $bar_should_be_hidden, $settings } from '../shared/state/mod';
import { MainContextMenu } from './ContextMenu';
import { ItemsDropableContainer } from './ItemsContainer';
import { componentByModule } from './mappins';

interface Container {
  id: string;
  items: ToolbarItem2[];
}

export function FancyToolbar() {
  const $dragging_id = useSignal<string | null>(null);
  const $containers = useComputed<Container[]>(() => [
    {
      id: 'left',
      items: $toolbar_state.value.left,
    },
    {
      id: 'center',
      items: $toolbar_state.value.center,
    },
    {
      id: 'right',
      items: $toolbar_state.value.right,
    },
  ]);

  const [openContextMenu, setOpenContextMenu] = useState(false);

  const focusedWindow = useSelector(Selectors.focused);

  const data = useBarData();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  const pointerSensor = useSensor(PointerSensor, {
    activationConstraint: {
      distance: 5,
    },
  });
  const sensors = useSensors(pointerSensor);

  function findContainer(id: string): Container | undefined {
    if (['left', 'center', 'right'].includes(id)) {
      return $containers.value.find((c) => c.id === id);
    }
    return $containers.value.find((c) =>
      c.items.some((item) => item === id || (typeof item === 'object' && item.id === id)),
    );
  }

  function handleDragStart({ active }: DragStartEvent) {
    $dragging_id.value = active.id as string;
  }

  // this handles the item container change while dragging
  function handleDragOver({ active, over }: DragOverEvent) {
    if (!over) return;

    const activeContainer = findContainer(active.id as string);
    const overContainer = findContainer(over.id as string);

    if (!activeContainer || !overContainer || activeContainer.id === overContainer.id) return;

    const activeItem = activeContainer.items.find((item) => item === active.id);
    if (!activeItem) return;

    const newOverContainerItems = [...overContainer.items];
    const overItemIdx = overContainer.items.findIndex((item) => item === over.id);
    if (overItemIdx !== -1) {
      newOverContainerItems.splice(overItemIdx, 0, activeItem);
    } else {
      newOverContainerItems.push(activeItem);
    }

    $toolbar_state.value = {
      ...$toolbar_state.value,
      [activeContainer.id]: activeContainer.items.filter((item) => item !== active.id),
      [overContainer.id]: newOverContainerItems,
    };
  }

  // this will handle the sorting
  function handleDragEnd({ active, over }: DragEndEvent) {
    if (!over || active.id === over.id) {
      $dragging_id.value = null;
      return;
    }

    const activeContainer = findContainer(active.id as string);
    const overContainer = findContainer(over.id as string);

    if (!activeContainer || !overContainer || activeContainer.id !== overContainer.id) {
      $dragging_id.value = null;
      return;
    }

    const activeIndex = activeContainer.items.findIndex((item) => item === active.id);
    const overIndex = overContainer.items.findIndex((item) => item === over.id);

    if (activeIndex !== -1 && overIndex !== -1) {
      const newItems = arrayMove(activeContainer.items, activeIndex, overIndex);
      $toolbar_state.value = {
        ...$toolbar_state.value,
        [activeContainer.id]: newItems,
      };
    }
  }

  const activeContainer = $dragging_id.value ? findContainer($dragging_id.value) : undefined;
  const draggingItem = activeContainer?.items.find((item) => item === $dragging_id.value);

  return (
    <AnimatedDropdown
      animationDescription={{
        openAnimationName: 'ft-bar-context-menu-open',
        closeAnimationName: 'ft-bar-context-menu-close',
      }}
      trigger={['contextMenu']}
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      dropdownRender={() => <MainContextMenu />}
    >
      <div
        className={cx('ft-bar', $settings.value.position.toLowerCase(), {
          'ft-bar-hidden': $bar_should_be_hidden.value,
        })}
        data-there-is-maximized-on-background={data.thereIsMaximizedOnBg}
        data-focused-is-maximized={!!focusedWindow?.isMaximized}
        data-focused-is-overlay={!!focusedWindow?.isSeelenOverlay}
        data-dynamic-color={$settings.value.dynamicColor}
      >
        <BackgroundByLayersV2 prefix="ft-bar" />
        <DndContext
          collisionDetection={closestCorners}
          onDragStart={handleDragStart}
          onDragOver={handleDragOver}
          onDragEnd={handleDragEnd}
          sensors={sensors}
        >
          {$containers.value.map(({ id, items }) => (
            <ItemsDropableContainer key={id} id={id} items={items} />
          ))}
          <DragOverlay>
            {draggingItem && componentByModule(draggingItem)}
          </DragOverlay>
        </DndContext>
      </div>
    </AnimatedDropdown>
  );
}

function useBarData() {
  const maximizedOnBg = useSelector((state: RootState) => {
    return state.openApps.find((app) => app.isZoomed && !app.isIconic);
  });

  const colors = useSelector(Selectors.windowColorByHandle, isEqual);
  const color = maximizedOnBg ? colors[String(maximizedOnBg.handle)] : undefined;

  useEffect(() => {
    if (color) {
      document.documentElement.style.setProperty(
        '--color-maximized-on-bg-background',
        color.background,
      );
      document.documentElement.style.setProperty(
        '--color-maximized-on-bg-foreground',
        color.foreground,
      );
    } else {
      document.documentElement.style.removeProperty('--color-maximized-on-bg-background');
      document.documentElement.style.removeProperty('--color-maximized-on-bg-foreground');
    }
  }, [color]);

  return {
    thereIsMaximizedOnBg: !!maximizedOnBg,
    dynamicBarColor: color,
  };
}
