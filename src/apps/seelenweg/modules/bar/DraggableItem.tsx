import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { HTMLAttributes, PropsWithChildren } from 'preact/compat';

import { SwItem } from '../shared/store/domain';

interface Props extends PropsWithChildren {
  item: SwItem;
}

export function DraggableItem({ children, item }: Props) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: item.id,
    animateLayoutChanges: () => false,
    disabled: item.type === 'Separator',
  });

  return (
    <div
      ref={setNodeRef}
      {...(attributes as HTMLAttributes<HTMLDivElement>)}
      {...listeners}
      style={{
        transform: CSS.Translate.toString(transform),
        transition,
        opacity: isDragging ? 0.3 : 1,
      }}
      className="weg-item-drag-container"
      /* onDragStart={() => {
        ref.current?.classList.add('dragging');
      }}
      onDragEnd={() => {
        setTimeout(() => {
          ref.current?.classList.remove('dragging');
        }, 150);
      }} */
    >
      {children}
    </div>
  );
}
