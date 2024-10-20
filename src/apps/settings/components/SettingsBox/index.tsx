import { ConfigProvider } from 'antd';

import cs from './index.module.css';

interface Props {
  children: React.ReactNode;
}

export const SettingsGroup = ({ children }: Props) => {
  return <div className={cs.group}>
    <div className={cs.content}>{children}</div>
  </div>;
};

interface SubGroupProps {
  children: React.ReactNode;
  label: React.ReactNode;
  disabled?: boolean;
}

export const SettingsSubGroup = ({ children, label, disabled }: SubGroupProps) => {
  return (
    <div className={cs.subgroup}>
      <div className={cs.subtitle}>{label}</div>
      <ConfigProvider componentDisabled={disabled}>
        <div className={cs.content}>{children}</div>
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
    <div className={cs.setting}>
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
