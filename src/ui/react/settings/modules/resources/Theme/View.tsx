import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button } from "antd";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { useSearchParams } from "react-router";

import { RootActions } from "../../shared/store/app/reducer.ts";
import type { RootState } from "../../shared/store/domain.ts";
import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";

import { ThemeConfigDefinition } from "./components/ThemeConfigDefinition.tsx";

export function ThemeView() {
  const { t } = useTranslation();
  const dispatch = useDispatch();

  const [searchParams] = useSearchParams();
  const id = searchParams.get("id");

  const theme = useSelector((state: RootState) => {
    return state.availableThemes.find((t) => t.id === id);
  });

  const handleReset = () => {
    dispatch(RootActions.resetThemeVariables({ themeId: theme!.id }));
  };

  if (!theme) {
    return <div>wow 404 !?</div>;
  }

  return (
    <>
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
