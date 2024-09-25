import InlineSVG from '../InlineSvg';
import cs from './index.module.css';

interface typesPropsIcon {
  iconName: string;
  size?: string | number;
  color?: string;
}

export function Icon(props: typesPropsIcon) {
  const { iconName, size, color, ...rest } = props;

  return (
    <InlineSVG
      {...rest}
      src={`../icons/${iconName}.svg`}
      className={cs.icon}
      style={{ height: size, color }}
    />
  );
}
