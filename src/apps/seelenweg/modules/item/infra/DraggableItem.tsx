import { Reorder } from 'framer-motion';
import { PropsWithChildren, useState } from 'react';

import { SwItem } from '../../shared/store/domain';

import { cx } from '../../../../shared/styles';

interface Props extends PropsWithChildren {
  item: SwItem;
  className?: String;
  drag: boolean | 'x' | 'y' | undefined;
}

export function DraggableItem({ children, item, className, drag }: Props) {
  const [ isDragging, setDragging ] = useState(false);

  return (
    <Reorder.Item
      as="div"
      value={item}
      drag={drag}
      className={cx('weg-item-drag-container', className, { 'dragging': isDragging })}
      onDragStart={() => {
        setDragging(true);
      }}
      onDragEnd={() => {
        setTimeout(() => {
          setDragging(false);
        }, 150);
      }}
    >
      {children}
    </Reorder.Item>
  );
}
