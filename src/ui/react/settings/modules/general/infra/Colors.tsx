import { Color, invoke, SeelenCommand } from "@seelen-ui/lib";
import { ColorPicker } from "antd";
import { useTranslation } from "react-i18next";

import { uiColors } from "../../../state/system.ts";

import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";
import cs from "./index.module.css";
import { useComputed } from "@preact/signals";

export function Colors() {
  const colors = useComputed(() => ({
    accent_light: new Color(uiColors.value.accent_light),
    accent_lighter: new Color(uiColors.value.accent_lighter),
    accent_lightest: new Color(uiColors.value.accent_lightest),
    accent: new Color(uiColors.value.accent),
    accent_dark: new Color(uiColors.value.accent_dark),
    accent_darker: new Color(uiColors.value.accent_darker),
    accent_darkest: new Color(uiColors.value.accent_darkest),
  })).value;

  const { t } = useTranslation();

  return (
    <SettingsGroup>
      <SettingsOption>
        <b>{t("general.accent_color")}</b>
        <div
          onClickCapture={(e) => {
            e.stopPropagation();
            invoke(SeelenCommand.OpenFile, { path: "ms-settings:colors" }).catch(console.error);
          }}
        >
          <ColorPicker value={colors.accent.toHexString()} disabledAlpha showText />
        </div>
      </SettingsOption>

      <div className={cs.palette}>
        <div style={{ backgroundColor: colors.accent_darkest.toHexString() }} />
        <div style={{ backgroundColor: colors.accent_darker.toHexString() }} />
        <div style={{ backgroundColor: colors.accent_dark.toHexString() }} />
        <div style={{ backgroundColor: colors.accent.toHexString() }} />
        <div style={{ backgroundColor: colors.accent_light.toHexString() }} />
        <div style={{ backgroundColor: colors.accent_lighter.toHexString() }} />
        <div style={{ backgroundColor: colors.accent_lightest.toHexString() }} />
      </div>
    </SettingsGroup>
  );
}
