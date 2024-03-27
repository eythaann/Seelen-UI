import { Tooltip } from 'antd';
import { PropsWithChildren } from 'react';

interface Props extends PropsWithChildren {
  showToltip: boolean;
  text: string;
}

export const TooltipWrap = ({ children, showToltip, text }: Props) => {
  if (!showToltip) {
    return children;
  }

  return <Tooltip title={text} placement="top" showArrow={false}>
    {children}
  </Tooltip>;
};