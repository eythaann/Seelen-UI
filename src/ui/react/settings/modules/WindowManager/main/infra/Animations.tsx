import { InputNumber, Select, Switch } from "antd";
import { useTranslation } from "react-i18next";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../../components/SettingsBox/index.tsx";

import { getWmConfig, setWmAnimations } from "../../application.ts";

export function WmAnimationsSettings() {
  const wmConfig = getWmConfig();
  const animations = wmConfig.animations;

  const { t } = useTranslation();

  return (
    <SettingsGroup>
      <SettingsSubGroup
        label={
          <SettingsOption
            label={t("wm.animations.enable")}
            action={
              <Switch
                checked={animations.enabled}
                onChange={(value) => {
                  setWmAnimations({
                    ...animations,
                    enabled: value,
                  });
                }}
              />
            }
          />
        }
      >
        <SettingsOption
          label={t("wm.animations.duration")}
          action={
            <InputNumber
              min={100}
              max={1500}
              value={Number(animations.durationMs)}
              onChange={(value) => {
                let parsed = value || 100;
                setWmAnimations({
                  ...animations,
                  durationMs: parsed,
                });
              }}
            />
          }
        />
        <SettingsOption
          label={t("wm.animations.ease_function")}
          action={
            <Select
              showSearch
              options={EaseFunctions}
              value={animations.easeFunction}
              onSelect={(value) => {
                setWmAnimations({
                  ...animations,
                  easeFunction: value,
                });
              }}
              style={{ width: "150px" }}
            />
          }
        />
      </SettingsSubGroup>
    </SettingsGroup>
  );
}

const EaseFunctions = [
  "Linear",
  "EaseIn",
  "EaseOut",
  "EaseInOut",
  "EaseInQuad",
  "EaseOutQuad",
  "EaseInOutQuad",
  "EaseInCubic",
  "EaseOutCubic",
  "EaseInOutCubic",
  "EaseInQuart",
  "EaseOutQuart",
  "EaseInOutQuart",
  "EaseInQuint",
  "EaseOutQuint",
  "EaseInOutQuint",
  "EaseInExpo",
  "EaseOutExpo",
  "EaseInOutExpo",
  "EaseInCirc",
  "EaseOutCirc",
  "EaseInOutCirc",
  "EaseInBack",
  "EaseOutBack",
  "EaseInOutBack",
  "EaseInElastic",
  "EaseOutElastic",
  "EaseInOutElastic",
  "EaseInBounce",
  "EaseOutBounce",
  "EaseInOutBounce",
].map((f) => ({ value: f }));
