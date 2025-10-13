import { FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { Icon } from "@shared/components/Icon";
import { Button, InputNumber, Select, Switch, Tooltip } from "antd";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";

import { newSelectors } from "../shared/store/app/reducer.ts";
import { RootSelectors } from "../shared/store/app/selectors.ts";
import { OptionsFromEnum } from "../shared/utils/app.ts";
import { FancyToolbarActions } from "./app.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";

export function FancyToolbarSettings() {
  const settings = useSelector(RootSelectors.fancyToolbar);
  const delayToShow = useSelector(newSelectors.fancyToolbar.delayToShow);
  const delayToHide = useSelector(newSelectors.fancyToolbar.delayToHide);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    dispatch(FancyToolbarActions.setEnabled(value));
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
              onChange={(value) => dispatch(FancyToolbarActions.setHeight(value || 0))}
              min={0}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("toolbar.dock_side")}</span>
            <Button.Group style={{ width: "60px" }}>
              {Object.values(FancyToolbarSide).map((side) => (
                <Button
                  key={side}
                  type={side === settings.position ? "primary" : "default"}
                  onClick={() => dispatch(FancyToolbarActions.setPosition(side))}
                >
                  <Icon iconName={`CgToolbar${side}`} size={18} />
                </Button>
              ))}
            </Button.Group>
          </SettingsOption>
          <SettingsOption>
            <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
              {t("toolbar.dynamic_color")}
              <Tooltip title={t("toolbar.dynamic_color_tooltip")}>
                <Icon iconName="LuCircleHelp" />
              </Tooltip>
            </span>
            <Switch
              checked={settings.dynamicColor}
              onChange={(value) => dispatch(FancyToolbarActions.setDynamicColor(value))}
            />
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
                onChange={(value) => dispatch(FancyToolbarActions.setHideMode(value))}
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
              onChange={(value) => {
                dispatch(FancyToolbarActions.setDelayToShow(value || 0));
              }}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("toolbar.delay_to_hide")} (ms)</span>
            <InputNumber
              value={delayToHide}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => {
                dispatch(FancyToolbarActions.setDelayToHide(value || 0));
              }}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("toolbar.show_hibernate")}</b>
          <Switch
            checked={settings.showHibernateButton}
            onChange={(value) => dispatch(FancyToolbarActions.setShowHibernateButton(value))}
          />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
