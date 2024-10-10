import { HTMLAttributes } from 'react';

import { cx } from '../../styles';
import InlineSVG from '../InlineSvg';
import cs from './index.module.css';

interface typesPropsIcon extends HTMLAttributes<HTMLElement> {
  iconName: string;
  size?: string | number;
  color?: string;
}

export function Icon(props: typesPropsIcon) {
  const { iconName, size, color, className, ...rest } = props;

  return (
    <InlineSVG
      {...rest}
      src={`../icons/${iconName}.svg`}
      className={cx('slu-icon', cs.icon, className)}
      style={{ height: size, color }}
    />
  );
}
