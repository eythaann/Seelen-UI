import { InputNumber } from "antd";
import { useTranslation } from "react-i18next";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../../../../components/SettingsBox/index.tsx";

import { getWmConfig, setWmFloatingSize } from "../../application.ts";

// bounds expressed as video-resolution shorthand: 380p (min) to 720p (max)
const MIN_WIDTH = 676;
const MIN_HEIGHT = 380;
const MAX_WIDTH = 1280;
const MAX_HEIGHT = 720;

export function WmFloatingSizeSettings() {
  const wmConfig = getWmConfig();
  const floating = wmConfig.floating;

  const { t } = useTranslation();

  return (
    <SettingsGroup>
      <SettingsSubGroup label={t("wm.floating.title")}>
        <SettingsOption
          label={t("wm.floating.width")}
          action={
            <InputNumber
              min={MIN_WIDTH}
              max={MAX_WIDTH}
              value={floating.width}
              onChange={(value) => {
                setWmFloatingSize({
                  ...floating,
                  width: value ?? MIN_WIDTH,
                });
              }}
            />
          }
        />
        <SettingsOption
          label={t("wm.floating.height")}
          action={
            <InputNumber
              min={MIN_HEIGHT}
              max={MAX_HEIGHT}
              value={floating.height}
              onChange={(value) => {
                setWmFloatingSize({
                  ...floating,
                  height: value ?? MIN_HEIGHT,
                });
              }}
            />
          }
        />
      </SettingsSubGroup>
    </SettingsGroup>
  );
}
