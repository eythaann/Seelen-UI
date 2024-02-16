import cs from './index.module.css';

interface Props {
  children: React.ReactNode;
}

export const SettingsBox = ({ children }: Props) => {
  return <div className={cs.box}>
    {children}
  </div>;
};