import { Tooltip, TooltipProps } from 'antd';
import { useEffect, useRef, useState } from 'react';

import { cx } from '../../styles';
import cs from './index.module.css';

interface Props {
  text: string;
  overlayClassName?: string;
  className?: string;
  placement?: TooltipProps['placement'];
  arrow?: TooltipProps['arrow'];
}

export function OverflowTooltip({ text, className, ...rest }: Props) {
  const textRef = useRef<HTMLSpanElement>(null);
  const [isOverflow, setIsOverflow] = useState(false);

  useEffect(() => {
    if (textRef.current) {
      setIsOverflow(textRef.current.scrollWidth > textRef.current.clientWidth);
    }
  }, [text]);

  const tooltip = isOverflow ? (
    <span dangerouslySetInnerHTML={{ __html: text.replaceAll(/\n/g, '<br />') }} />
  ) : null;

  return (
    <Tooltip title={tooltip} {...rest}>
      <span ref={textRef} className={cx(cs.text, className)}>
        {text}
      </span>
    </Tooltip>
  );
}
