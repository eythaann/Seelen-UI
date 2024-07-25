import { Tooltip } from 'antd';
import { useEffect, useRef, useState } from 'react';

import cs from './index.module.css';

interface Props {
  text: string;
}

export function OverflowTooltip({ text }: Props) {
  const textRef = useRef<HTMLSpanElement>(null);
  const [isOverflow, setIsOverflow] = useState(false);

  useEffect(() => {
    if (textRef.current) {
      setIsOverflow(textRef.current.scrollWidth > textRef.current.clientWidth);
    }
  }, [text]);

  return (
    <Tooltip title={isOverflow ? text : undefined}>
      <span ref={textRef} className={cs.text}>
        {text}
      </span>
    </Tooltip>
  );
}
