import {
  closestCorners,
  DndContext,
  DragEndEvent,
  DragOverEvent,
  DragOverlay,
  DragStartEvent,
  PointerSensor,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useSignal } from '@preact/signals';
import { throttle } from 'lodash';
import { ComponentChildren } from 'preact';
import { useMemo } from 'preact/hooks';

import { genericHandleDragOver } from '../DndKit/utils';
import cs from './index.module.css';

interface Props<T> {
  options: { label: ComponentChildren; value: T }[];
  enabled: T[];
  onChange: (enabled: T[]) => void;
}

export function VerticalSortableSelect<T extends string>({ options, enabled, onChange }: Props<T>) {
  const enabledOpts = options
    .filter(({ value }) => enabled.includes(value))
    .toSorted((a, b) => enabled.indexOf(a.value) - enabled.indexOf(b.value));
  const disabledOpts = options.filter(({ value }) => !enabled.includes(value));

  const containers = [
    {
      id: 'enabled' as T,
      items: enabledOpts.map(({ value }) => value),
    },
    {
      id: 'disabled' as T,
      items: disabledOpts.map(({ value }) => value),
    },
  ];

  const $dragging_id = useSignal<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    }),
  );

  function handleDragStart(e: DragStartEvent) {
    $dragging_id.value = e.active.id as string;
  }

  const _handleDragOver = useMemo(() => throttle(genericHandleDragOver<T>, 100), []);
  function handleDragOver(e: DragOverEvent) {
    _handleDragOver(e, containers, (newContainers) => {
      const enabledIds = newContainers.find((c) => c.id === 'enabled')?.items ?? [];
      onChange(enabledIds);
    });
  }

  function handleDragEnd(e: DragEndEvent) {
    const { active, over } = e;
    if (!over || active.id === over.id) {
      $dragging_id.value = null;
      return;
    }

    const oldPos = enabled.indexOf(active.id as T);
    const newPos = enabled.indexOf(over.id as T);
    const newEnabled = arrayMove(enabled, oldPos, newPos);
    onChange(newEnabled);
  }

  const draggingOption = options.find(({ value }) => value === $dragging_id.value) ?? null;
  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
      onDragCancel={() => ($dragging_id.value = null)}
      collisionDetection={closestCorners}
    >
      <div className={cs.container}>
        {containers.map(({ id, items }) => (
          <div className={cs.box}>
            <div className={cs.header}>{id === 'enabled' ? 'Enabled' : 'Disabled'}</div>
            <DndDropableAndSortableContainer key={id} id={id} items={items} className={cs.list}>
              {items.map((id) => (
                <Entry key={id} value={id}>
                  <div className={cs.item}>{options.find(({ value }) => value === id)?.label}</div>
                </Entry>
              ))}
            </DndDropableAndSortableContainer>
          </div>
        ))}
        <DragOverlay>
          {draggingOption && <div className={cs.item}>{draggingOption.label}</div>}
        </DragOverlay>
      </div>
    </DndContext>
  );
}

function DndDropableAndSortableContainer({
  id,
  items,
  className,
  children,
}: {
  id: string;
  items: string[];
  className?: string;
  children: ComponentChildren;
}) {
  const { setNodeRef } = useDroppable({ id });

  return (
    <SortableContext items={items} strategy={verticalListSortingStrategy}>
      <div ref={setNodeRef} className={className}>
        {children}
      </div>
    </SortableContext>
  );
}

function Entry({ value, children }: { value: string; children: ComponentChildren }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: value,
    animateLayoutChanges: () => false,
  });

  return (
    <div
      ref={setNodeRef}
      {...(attributes as any)}
      {...listeners}
      style={{
        transform: CSS.Translate.toString(transform),
        transition,
        opacity: isDragging ? 0.1 : 1,
      }}
    >
      {children}
    </div>
  );
}
