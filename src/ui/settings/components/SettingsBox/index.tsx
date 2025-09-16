import { Icon } from "@shared/components/Icon";
import { ConfigProvider, Tooltip } from "antd";
import { ComponentChildren } from "preact";

import cs from "./index.module.css";

interface Props {
  children: React.ReactNode;
}

export const SettingsGroup = ({ children }: Props) => {
  return (
    <div className={cs.group}>
      <div className={cs.content}>{children}</div>
    </div>
  );
};

interface SubGroupProps {
  children: React.ReactNode;
  label: React.ReactNode;
  disabled?: boolean;
}

export const SettingsSubGroup = (
  { children, label, disabled }: SubGroupProps,
) => {
  return (
    <div className={cs.subgroup}>
      <div className={cs.subtitle}>{label}</div>
      <ConfigProvider componentDisabled={disabled}>
        <div className={cs.content}>{children}</div>
      </ConfigProvider>
    </div>
  );
};

type OptionProps = {
  label?: ComponentChildren;
  tip?: ComponentChildren;
  description?: ComponentChildren;
  action?: ComponentChildren;
  children?: ComponentChildren;
};

export const SettingsOption = (props: OptionProps) => {
  return (
    <div className={cs.setting}>
      {props.children
        ? (
          props.children
        )
        : (
          <>
            <div className={cs.info}>
              <div className={cs.label}>
                {props.label}
                {props.tip && (
                  <Tooltip title={props.tip}>
                    <Icon iconName="HiOutlineInformationCircle" />
                  </Tooltip>
                )}
              </div>
              {props.description && <div className={cs.description}>{props.description}</div>}
            </div>
            <div className={cs.action}>
              {props.action}
            </div>
          </>
        )}
    </div>
  );
};
