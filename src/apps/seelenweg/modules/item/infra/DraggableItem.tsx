import { Reorder } from 'framer-motion';
import { PropsWithChildren, useRef } from 'react';

import { SwItem } from '../../shared/store/domain';

interface Props extends PropsWithChildren {
  item: SwItem;
}

export function DraggableItem({ children, item }: Props) {
  const isDragging = useRef(false);

  return (
    <Reorder.Item
      as="div"
      value={item}
      className="weg-item-drag-container"
      onDragStart={() => {
        isDragging.current = true;
      }}
      onDragEnd={() => {
        setTimeout(() => {
          isDragging.current = false;
        }, 150);
      }}
      onClickCapture={(e) => {
        if (isDragging.current) {
          e.stopPropagation();
        }
      }}
    >
      {children}
    </Reorder.Item>
  );
}
