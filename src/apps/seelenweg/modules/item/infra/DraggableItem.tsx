import { Reorder } from 'framer-motion';
import { PropsWithChildren, useRef } from 'react';

import { SwItem } from '../../shared/store/domain';

interface Props extends PropsWithChildren {
  item: SwItem;
}

export function DraggableItem({ children, item }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  return (
    <Reorder.Item
      as="div"
      ref={ref}
      value={item}
      drag
      className="weg-item-drag-container"
      onDragStart={() => {
        ref.current?.classList.add('dragging');
      }}
      onDragEnd={() => {
        setTimeout(() => {
          ref.current?.classList.remove('dragging');
        }, 150);
      }}
    >
      {children}
    </Reorder.Item>
  );
}
