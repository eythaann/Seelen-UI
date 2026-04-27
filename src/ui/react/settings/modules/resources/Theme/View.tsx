import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Button } from "antd";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";

import { ThemeConfigDefinition } from "./components/ThemeConfigDefinition.tsx";
import { ResourceDescription } from "../ResourceCard.tsx";
import { resetThemeVariables } from "./application.ts";
import { themes, widgets } from "../../../state/resources.ts";
import cs from "../infra.module.css";

export function ThemeView() {
  const { t } = useTranslation();

  const [searchParams] = useSearchParams();
  const id = searchParams.get("id");

  const theme = themes.value.find((t) => t.id === id);

  const handleReset = () => {
    resetThemeVariables(theme!.id);
  };

  if (!theme) {
    return <div>wow 404 !?</div>;
  }

  const affectedWidgets = Object.keys(theme.styles)
    .map((widgetId) => widgets.value.find((w) => w.id === widgetId)!)
    .filter(Boolean);

  return (
    <>
      <SettingsGroup>
        <ResourceDescription text={theme.metadata.description} />
      </SettingsGroup>

      {affectedWidgets.length > 0 && (
        <SettingsGroup>
          <SettingsSubGroup label={t("resources.affected_widgets")}>
            <div className={cs.tags}>
              {affectedWidgets.map((widget) => (
                <div key={widget.id} className={cs.tag}>
                  <ResourceText text={widget.metadata.displayName} />
                </div>
              ))}
            </div>
          </SettingsSubGroup>
        </SettingsGroup>
      )}

      <SettingsGroup>
        <SettingsOption
          label={t("reset_all_to_default")}
          action={
            <Button onClick={handleReset}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          }
        />
      </SettingsGroup>

      {theme.settings.map((def, idx) => <ThemeConfigDefinition key={idx} themeId={theme.id} def={def} />)}
    </>
  );
}
