import { ConfigProvider } from 'antd';

import cs from './index.module.css';

interface Props {
  children: React.ReactNode;
}

export const SettingsGroup = ({ children }: Props) => {
  return <div className={cs.group}>
    <div className={cs.blur}/>
    <div className={cs.noise}/>
    <div className={cs.content}>{children}</div>
  </div>;
};

interface SubGroupProps {
  children: React.ReactNode;
  label: React.ReactNode;
  disableOptions?: boolean;
}

export const SettingsSubGroup = ({ children, label, disableOptions }: SubGroupProps) => {
  return (
    <div>
      <div className={cs.subtitle}>{label}</div>
      <ConfigProvider componentDisabled={disableOptions}>
        <div className={cs.subgroup}>{children}</div>
      </ConfigProvider>
    </div>
  );
};

type OptionProps =
  | {
    children: React.ReactNode;
  }
  | {
    label: React.ReactNode;
    trigger: React.ReactNode;
  };

export const SettingsOption = (props: OptionProps) => {
  return (
    <div className={cs.box}>
      {'children' in props ? (
        props.children
      ) : (
        <>
          <span>{props.label}</span>
          {props.trigger}
        </>
      )}
    </div>
  );
};
