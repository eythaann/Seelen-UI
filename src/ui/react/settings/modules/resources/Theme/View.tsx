import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button } from "antd";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router";

import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";

import { ThemeConfigDefinition } from "./components/ThemeConfigDefinition.tsx";
import { ResourceTextAsMarkdown } from "libs/ui/react/components/ResourceText/index.tsx";
import { resetThemeVariables } from "./application.ts";
import { themes } from "../../../state/resources.ts";

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

  return (
    <>
      <SettingsGroup>
        <ResourceTextAsMarkdown text={theme.metadata.description} />
      </SettingsGroup>

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
