import cs from './index.module.css';

interface Props {
  children: React.ReactNode;
}

export const SettingsGroup = ({ children }: Props) => {
  return <div className={cs.group}>
    {children}
  </div>;
};

interface SubGroupProps {
  children: React.ReactNode;
  label: React.ReactNode;
}

export const SettingsSubGroup = ({ children, label }: SubGroupProps) => {
  return <div>
    <div className={cs.subtitle}>{label}</div>
    <div className={cs.subgroup}>
      {children}
    </div>
  </div>;
};

export const SettingsOption = ({ children }: Props) => {
  return <div className={cs.box}>
    {children}
  </div>;
};