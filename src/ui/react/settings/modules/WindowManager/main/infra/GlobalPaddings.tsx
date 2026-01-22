import type { Rect } from "@seelen-ui/lib";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, InputNumber } from "antd";
import { useTranslation } from "react-i18next";

import { getWmConfig, setWmWorkspaceGap, setWmWorkspaceMargin, setWmWorkspacePadding } from "../../application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../../components/SettingsBox/index.tsx";

export const GlobalPaddings = () => {
  const wmConfig = getWmConfig();
  const workspaceGap = wmConfig.workspaceGap;
  const workspacePadding = wmConfig.workspacePadding;
  const workAreaOffset = wmConfig.workspaceMargin;

  const onChangeGlobalOffset = (side: keyof Rect, value: number | null) => {
    setWmWorkspaceMargin({
      ...workAreaOffset,
      [side]: Math.round(value || 0),
    });
  };

  const onChangeDefaultGap = (value: number | null) => {
    setWmWorkspaceGap(Math.round(value || 0));
  };

  const onChangeDefaultPadding = (value: number | null) => {
    setWmWorkspacePadding(Math.round(value || 0));
  };

  return (
    <WindowManagerSpacingSettings
      gap={workspaceGap}
      padding={workspacePadding}
      margins={workAreaOffset}
      onChangeGap={onChangeDefaultGap}
      onChangePadding={onChangeDefaultPadding}
      onChangeMargins={onChangeGlobalOffset}
    />
  );
};

interface WindowManagerSpacingSettings {
  gap: number | null;
  padding: number | null;
  margins: Rect | null;
  onChangeGap: (v: number | null) => void;
  onChangePadding: (v: number | null) => void;
  onChangeMargins: (side: keyof Rect, value: number | null) => void;
  onClear?: () => void;
}

export function WindowManagerSpacingSettings(
  props: WindowManagerSpacingSettings,
) {
  const {
    gap,
    padding,
    margins,
    onChangeGap,
    onChangePadding,
    onChangeMargins,
    onClear,
  } = props;

  const { t } = useTranslation();

  return (
    <SettingsGroup>
      {onClear && (
        <SettingsOption>
          <span>{t("header.labels.seelen_wm")}</span>
          <Button onClick={onClear}>
            <Icon iconName="IoTrash" size={14} />
          </Button>
        </SettingsOption>
      )}
      <SettingsOption>
        <b>{t("wm.space_between_containers")}</b>
        <InputNumber
          value={gap}
          onChange={onChangeGap}
          min={0}
          placeholder={t("inherit")}
        />
      </SettingsOption>
      <SettingsOption>
        <b>{t("wm.workspace_padding")}</b>
        <InputNumber
          value={padding}
          onChange={onChangePadding}
          min={0}
          placeholder={t("inherit")}
        />
      </SettingsOption>
      <SettingsSubGroup label={t("wm.workspace_offset")}>
        <SettingsOption>
          <span>{t("sides.left")}</span>
          <InputNumber
            value={margins?.left}
            onChange={onChangeMargins.bind(null, "left")}
            min={0}
            placeholder={t("inherit")}
          />
        </SettingsOption>
        <SettingsOption>
          <span>{t("sides.top")}</span>
          <InputNumber
            value={margins?.top}
            onChange={onChangeMargins.bind(null, "top")}
            min={0}
            placeholder={t("inherit")}
          />
        </SettingsOption>
        <SettingsOption>
          <span>{t("sides.right")}</span>
          <InputNumber
            value={margins?.right}
            onChange={onChangeMargins.bind(null, "right")}
            min={0}
            placeholder={t("inherit")}
          />
        </SettingsOption>
        <SettingsOption>
          <span>{t("sides.bottom")}</span>
          <InputNumber
            value={margins?.bottom}
            onChange={onChangeMargins.bind(null, "bottom")}
            min={0}
            placeholder={t("inherit")}
          />
        </SettingsOption>
      </SettingsSubGroup>
    </SettingsGroup>
  );
}
