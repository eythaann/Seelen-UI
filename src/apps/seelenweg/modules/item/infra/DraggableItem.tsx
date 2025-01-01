import { Reorder } from 'framer-motion';
import { PropsWithChildren, useRef, useState } from 'react';

import { SwItem } from '../../shared/store/domain';

import { cx } from '../../../../shared/styles';

interface Props extends PropsWithChildren {
  item: SwItem;
  className?: String;
  drag: boolean | 'x' | 'y' | undefined;
}

export function DraggableItem({ children, item, className, drag }: Props) {
  const [ isDragging, setDragging ] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  return (
    <Reorder.Item
      as="div"
      ref={ref}
      value={item}
      drag={drag}
      className={cx('weg-item-drag-container', className, { 'dragging': isDragging })}
      onDragStart={() => {
        ref.current?.classList.add('dragging');
        setDragging(true);
      }}
      onDragEnd={() => {
        setTimeout(() => {
          ref.current?.classList.remove('dragging');
          setDragging(false);
        }, 150);
      }}
    >
      {children}
    </Reorder.Item>
  );
}
