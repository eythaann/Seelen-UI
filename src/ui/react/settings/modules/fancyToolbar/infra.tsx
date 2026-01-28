import { FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, InputNumber, Select, Switch } from "antd";
import { useTranslation } from "react-i18next";

import { OptionsFromEnum } from "../shared/utils/app.ts";
import {
  getToolbarConfig,
  setToolbarDelayToHide,
  setToolbarDelayToShow,
  setToolbarEnabled,
  setToolbarHeight,
  setToolbarHideMode,
  setToolbarPosition,
} from "./application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";
import Compact from "antd/es/space/Compact";

export function FancyToolbarSettings() {
  const settings = getToolbarConfig();
  const delayToShow = settings.delayToShow;
  const delayToHide = settings.delayToHide;

  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    setToolbarEnabled(value);
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("toolbar.enable")}</b>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("toolbar.label")}>
          <SettingsOption>
            <span>{t("toolbar.height")}</span>
            <InputNumber
              value={settings.height}
              onChange={(value) => setToolbarHeight(value || 0)}
              min={0}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("toolbar.dock_side")}</span>
            <Compact>
              {Object.values(FancyToolbarSide).map((side) => (
                <Button
                  key={side}
                  type={side === settings.position ? "primary" : "default"}
                  onClick={() => setToolbarPosition(side)}
                >
                  <Icon iconName={`CgToolbar${side}`} size={18} />
                </Button>
              ))}
            </Compact>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption>
              <b>{t("toolbar.auto_hide")}</b>
              <Select
                style={{ width: "120px" }}
                value={settings.hideMode}
                options={OptionsFromEnum(t, HideMode, "toolbar.hide_mode")}
                onChange={(value) => setToolbarHideMode(value)}
              />
            </SettingsOption>
          }
        >
          <SettingsOption>
            <span>{t("toolbar.delay_to_show")} (ms)</span>
            <InputNumber
              value={delayToShow}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => setToolbarDelayToShow(value || 0)}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("toolbar.delay_to_hide")} (ms)</span>
            <InputNumber
              value={delayToHide}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => setToolbarDelayToHide(value || 0)}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
}
