import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { path } from "@tauri-apps/api";
import { Button, Input, Select } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { resolveDataPath } from "../shared/config/infra.ts";

import { LoadCustomConfigFile, simulatePerm } from "./application.ts";
import { createSampleDialog } from "./simulateDialogConfig.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";

export function DevToolsSettings() {
  const [simWidgetId, setSimWidgetId] = useState("");
  const [simPerm, setSimPerm] = useState<string>("run");
  const [simResult, setSimResult] = useState<string | null>(null);

  const { t } = useTranslation();

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

  async function onSimulatePerm() {
    setSimResult(null);
    try {
      await simulatePerm(simWidgetId, simPerm);
      setSimResult(t("devtools.simulate_perm.result_allowed"));
    } catch {
      setSimResult(t("devtools.simulate_perm.result_denied"));
    }
  }

  async function onSimulateDialog() {
    await invoke(SeelenCommand.TriggerDialog, { dialog: createSampleDialog() });
  }

  return (
    <>
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

      <SettingsGroup>
        <SettingsOption>
          <span>{t("devtools.simulate_dialog.label")}</span>
          <Button onClick={onSimulateDialog}>{t("devtools.simulate_dialog.trigger")}</Button>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("devtools.simulate_perm.label")}>
          <SettingsOption>
            <Input
              value={simWidgetId}
              onChange={(e) => {
                setSimWidgetId(e.currentTarget.value);
                setSimResult(null);
              }}
              placeholder={t("devtools.simulate_perm.widget_id_placeholder")}
            />
          </SettingsOption>
          <SettingsOption>
            <Select
              value={simPerm}
              onChange={(v) => {
                setSimPerm(v);
                setSimResult(null);
              }}
              options={[{ value: "run" }, { value: "open_file" }]}
            />
            <Button onClick={onSimulatePerm} disabled={!simWidgetId}>
              {t("devtools.simulate_perm.trigger")}
            </Button>
          </SettingsOption>
          {simResult && (
            <SettingsOption>
              <span>{simResult}</span>
            </SettingsOption>
          )}
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
}
