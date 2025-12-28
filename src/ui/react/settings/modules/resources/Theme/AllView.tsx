import { SeelenCommand } from "@seelen-ui/lib";
import { ResourceKind, type Theme, type ThemeId } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { path } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/core";
import { Button, Switch } from "antd";
import { Reorder } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { RootActions } from "../../shared/store/app/reducer.ts";
import { RootSelectors } from "../../shared/store/app/selectors.ts";

import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";
import { ResourceCard } from "../ResourceCard.tsx";

export function ThemesView() {
  const activeIds = useSelector(RootSelectors.activeThemes);
  const allThemes = useSelector(RootSelectors.availableThemes);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function toggleTheme(themeId: ThemeId) {
    if (activeIds.includes(themeId)) {
      dispatch(
        RootActions.setSelectedThemes(activeIds.filter((x) => x !== themeId)),
      );
    } else {
      dispatch(RootActions.setSelectedThemes([...activeIds, themeId]));
    }
  }

  function onReorder(themes: ThemeId[]) {
    dispatch(RootActions.setSelectedThemes(themes));
  }

  const disabled: Theme[] = [];
  const enabled: Theme[] = [];
  for (const theme of allThemes) {
    if (activeIds.includes(theme.id)) {
      enabled.push(theme);
    } else {
      disabled.push(theme);
    }
  }
  enabled.sort((a, b) => activeIds.indexOf(a.id) - activeIds.indexOf(b.id));

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("resources.open_folder")}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, {
                path: await path.join(dataDir, "themes"),
              });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t("resources.discover")}:</span>
          <Button
            href="https://seelen.io/resources/s?category=Theme"
            target="_blank"
            type="link"
          >
            https://seelen.io/resources/s?category=Theme
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        <b>{t("general.theme.selected")}</b>
        <Reorder.Group
          values={activeIds}
          onReorder={onReorder}
          className={cs.reorderGroup}
        >
          {enabled.map((theme) => (
            <Reorder.Item key={theme.id} value={theme.id}>
              <ThemeItem
                key={theme.id}
                theme={theme}
                onToggle={() => toggleTheme(theme.id)}
                checked
              />
            </Reorder.Item>
          ))}
        </Reorder.Group>

        <b>{t("general.theme.available")}</b>
        {disabled.map((theme) => (
          <ThemeItem
            key={theme.id}
            theme={theme}
            onToggle={() => toggleTheme(theme.id)}
            checked={false}
          />
        ))}
      </div>
    </>
  );
}

interface ThemeItemProps {
  theme: Theme;
  onToggle: () => void;
  checked: boolean;
}

function ThemeItem({ theme, checked, onToggle }: ThemeItemProps) {
  return (
    <ResourceCard
      resource={theme}
      kind={ResourceKind.Theme}
      actions={
        <>
          {theme.settings.length > 0 && (
            <NavLink to={`/theme/${theme.id.replace("@", "")}`}>
              <Button type="text">
                <Icon iconName="RiSettings4Fill" />
              </Button>
            </NavLink>
          )}
          {theme.id !== "@default/theme" && <Switch value={checked} onChange={onToggle} />}
        </>
      }
    />
  );
}
