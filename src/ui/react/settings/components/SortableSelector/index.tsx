import { DragDropProvider, DragOverlay, useDroppable } from "@dnd-kit/react";
import { useSortable } from "@dnd-kit/react/sortable";
import { arrayMove } from "@dnd-kit/helpers";
import { throttle } from "lodash";
import type { ComponentChildren } from "preact";
import { useMemo } from "preact/hooks";
import { genericHandleDragOver } from "../../../../../../libs/ui/react/utils/DndKit/utils.ts";

import cs from "./index.module.css";

interface Props<T> {
  disabled?: boolean;
  options: { label: ComponentChildren; value: T }[];
  enabled: T[];
  onChange: (enabled: T[]) => void;
}

export function VerticalSortableSelect<T extends string>({
  options,
  enabled,
  onChange,
  disabled = false,
}: Props<T>) {
  const enabledOpts = options
    .filter(({ value }) => enabled.includes(value))
    .toSorted((a, b) => enabled.indexOf(a.value) - enabled.indexOf(b.value));
  const disabledOpts = options.filter(({ value }) => !enabled.includes(value));

  const containers = [
    {
      id: "enabled" as T,
      items: enabledOpts.map(({ value }) => value),
    },
    {
      id: "disabled" as T,
      items: disabledOpts.map(({ value }) => value),
    },
  ];

  const _handleDragOver = useMemo(() => throttle(genericHandleDragOver<T>, 100), []);
  function handleDragOver(event: any) {
    _handleDragOver(event, containers, (newContainers) => {
      const enabledIds = newContainers.find((c) => c.id === "enabled")?.items ?? [];
      onChange(enabledIds);
    });
  }

  function handleDragEnd(event: any) {
    const { source, target } = event.operation;
    if (!target || source.id === target.id || event.canceled) return;

    const oldPos = enabled.indexOf(source.id as T);
    const newPos = enabled.indexOf(target.id as T);
    const newEnabled = arrayMove(enabled, oldPos, newPos).filter(Boolean);
    onChange(newEnabled);
  }

  return (
    <DragDropProvider
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
    >
      <div className={cs.container}>
        {containers.map(({ id, items }) => (
          <div className={cs.box}>
            <div className={cs.header}>{id === "enabled" ? "Enabled" : "Disabled"}</div>
            <DndDropableAndSortableContainer key={id} id={id} items={items} className={cs.list}>
              {items.map((id, index) => (
                <Entry key={id} value={id} disabled={disabled} index={index}>
                  <div className={cs.item}>{options.find(({ value }) => value === id)?.label}</div>
                </Entry>
              ))}
            </DndDropableAndSortableContainer>
          </div>
        ))}
        <DragOverlay>
          {(source) => {
            const opt = options.find(({ value }) => value === source.id);
            return opt ? <div className={cs.item}>{opt.label}</div> : null;
          }}
        </DragOverlay>
      </div>
    </DragDropProvider>
  );
}

function DndDropableAndSortableContainer({
  id,
  className,
  children,
}: {
  id: string;
  items: string[];
  className?: string;
  children: ComponentChildren;
}) {
  const droppable = useDroppable({ id });

  return (
    <div ref={droppable.ref} className={className}>
      {children}
    </div>
  );
}

function Entry({
  value,
  children,
  disabled,
  index,
}: {
  value: string;
  children: ComponentChildren;
  disabled: boolean;
  index: number;
}) {
  const sortable = useSortable({
    id: value,
    index,
    disabled,
  });

  let opacity = 1;

  if (sortable.isDragging) {
    opacity = 0.1;
  }

  if (disabled) {
    opacity = 0.6;
  }

  return (
    <div ref={sortable.ref} style={{ opacity }}>
      {children}
    </div>
  );
}
