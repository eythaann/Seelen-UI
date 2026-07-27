import { BorderSettings } from "../../border/infra.tsx";

import { InputNumber, Select, Switch } from "antd";
import { useTranslation } from "react-i18next";

import {
  getWmConfig,
  patchWmConfig,
  setWmDragBehavior,
  setWmResizeDelta,
  setWmStackBarVisibility,
} from "../../application.ts";

import { SettingsGroup, SettingsOption } from "../../../../../../components/SettingsBox/index.tsx";
import { WmDragBehavior, WmStackBarVisibility } from "@seelen-ui/lib/types";

import { WmAnimationsSettings } from "./Animations.tsx";
import { GlobalPaddings } from "./GlobalPaddings.tsx";
import { LayoutSelector } from "./LayoutSelector.tsx";

export function WindowManagerSettings() {
  const wmConfig = getWmConfig();
  const resizeDelta = wmConfig.resizeDelta;
  const dragBehavior = wmConfig.dragBehavior;
  const stackBarVisibility = wmConfig.stackBarVisibility;

  const { t } = useTranslation();

  const onChangeResizeDelta = (value: number | null) => {
    setWmResizeDelta(value || 0);
  };

  const onChangeDragBehavior = (value: WmDragBehavior) => {
    setWmDragBehavior(value);
  };

  const onChangeStackBarVisibility = (value: WmStackBarVisibility) => {
    setWmStackBarVisibility(value);
  };

  const setWmAutoStack = (value: boolean) => {
    patchWmConfig({ autoStackingByCategory: value });
  };

  return (
    <>
      <LayoutSelector />

      <SettingsGroup>
        <SettingsOption>
          <b>{t("wm.auto_stack")}</b>
          <Switch checked={wmConfig.autoStackingByCategory} onChange={setWmAutoStack} />
        </SettingsOption>

        <SettingsOption>
          <b>{t("wm.stack_bar_visibility")}</b>
          <Select
            style={{ width: "200px" }}
            value={stackBarVisibility}
            options={[
              {
                label: t("wm.stack_bar_visibility_options.always"),
                value: WmStackBarVisibility.Always,
              },
              {
                label: t("wm.stack_bar_visibility_options.as_needed"),
                value: WmStackBarVisibility.AsNeeded,
              },
            ]}
            onSelect={onChangeStackBarVisibility}
          />
        </SettingsOption>
      </SettingsGroup>

      <GlobalPaddings />
      <BorderSettings />

      <SettingsGroup>
        <SettingsOption>
          <b>{t("wm.drag_behavior")}</b>
          <Select
            style={{ width: "200px" }}
            value={dragBehavior}
            options={[
              {
                label: t("wm.drag_behavior_options.sort"),
                value: WmDragBehavior.Sort,
              },
              {
                label: t("wm.drag_behavior_options.swap"),
                value: WmDragBehavior.Swap,
              },
            ]}
            onSelect={onChangeDragBehavior}
          />
        </SettingsOption>
      </SettingsGroup>
      <WmAnimationsSettings />

      <SettingsGroup>
        <SettingsOption>
          <b>{t("wm.resize_delta")}</b>
          <InputNumber value={resizeDelta} onChange={onChangeResizeDelta} min={1} max={40} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
