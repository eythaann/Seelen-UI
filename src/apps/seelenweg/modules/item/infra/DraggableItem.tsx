import { Reorder } from 'framer-motion';
import { PropsWithChildren, useRef } from 'react';

import { SwItem } from '../../shared/store/domain';

import { cx } from '../../../../shared/styles';

interface Props extends PropsWithChildren {
  item: SwItem;
  className?: String;
}

export function DraggableItem({ children, item, className }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  return (
    <Reorder.Item
      as="div"
      ref={ref}
      value={item}
      drag
      className={cx('weg-item-drag-container', className)}
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
