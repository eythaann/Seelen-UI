import { InputNumber, Switch } from "antd";
import { useTranslation } from "react-i18next";

import { getBorderConfig, setBorderEnabled, setBorderOffset, setBorderWidth } from "./application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";

export const BorderSettings = () => {
  const borderConfig = getBorderConfig();
  const enabled = borderConfig.enabled;
  const offset = borderConfig.offset;
  const width = borderConfig.width;

  const { t } = useTranslation();

  const toggleEnabled = (value: boolean) => {
    setBorderEnabled(value);
  };

  const updateOffset = (value: number | null) => {
    setBorderOffset(value || 0);
  };

  const updateWidth = (value: number | null) => {
    setBorderWidth(value || 0);
  };

  return (
    <SettingsGroup>
      <SettingsSubGroup
        label={
          <SettingsOption>
            <span>{t("wm.border.enable")}</span>
            <Switch value={enabled} onChange={toggleEnabled} />
          </SettingsOption>
        }
      >
        <SettingsOption>
          <span>{t("wm.border.offset")}</span>
          <InputNumber value={offset} onChange={updateOffset} />
        </SettingsOption>
        <SettingsOption>
          <span>{t("wm.border.width")}</span>
          <InputNumber value={width} onChange={updateWidth} />
        </SettingsOption>
      </SettingsSubGroup>
    </SettingsGroup>
  );
};
