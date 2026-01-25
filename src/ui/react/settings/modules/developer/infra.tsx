import { SeelenCommand } from "@seelen-ui/lib";
import { path } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/core";
import { Button, Switch } from "antd";
import { useTranslation } from "react-i18next";

import { resolveDataPath } from "../shared/config/infra.ts";

import { getWegConfig, patchWegConfig } from "../seelenweg/application.ts";
import { getDevTools, LoadCustomConfigFile, setDevTools } from "./application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";

export function DeveloperTools() {
  const devTools = getDevTools();
  const showEndTask = getWegConfig().showEndTask;

  const { t } = useTranslation();

  function onToggleDevTools(value: boolean) {
    setDevTools(value);
  }

  async function openSettingsFile() {
    invoke(SeelenCommand.OpenFile, {
      path: await resolveDataPath("settings.json"),
    });
  }

  async function openInstallFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.resourceDir() });
  }

  async function openDataFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.appDataDir() });
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("devtools.enable")}</b>
          <Switch value={devTools} onChange={onToggleDevTools} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("weg.show_end_task")}</b>
          <Switch
            checked={showEndTask}
            onChange={(value) => patchWegConfig({ showEndTask: value })}
          />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("devtools.app_folders")}>
          <SettingsOption>
            <span>{t("devtools.install_folder")}</span>
            <Button onClick={openInstallFolder}>{t("open")}</Button>
          </SettingsOption>
          <SettingsOption>
            <span>{t("devtools.data_folder")}</span>
            <Button onClick={openDataFolder}>{t("open")}</Button>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>{t("devtools.settings_file")}</span>
          <Button onClick={openSettingsFile}>{t("open")}</Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t("devtools.custom_config_file")}:</span>
          <Button onClick={LoadCustomConfigFile}>{t("devtools.load")}</Button>
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
