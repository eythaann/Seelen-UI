import { Reorder } from 'framer-motion';
import { useRef } from 'react';

type _props = Parameters<typeof Reorder.Item>[0];

interface Props extends _props {}

export function DraggableItem({ children, ...rest }: Props) {
  const isDragging = useRef(false);

  return (
    <Reorder.Item
      {...rest}
      as="div"
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
