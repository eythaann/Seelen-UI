import { createSelector } from "@reduxjs/toolkit";
import { type AppConfig, AppExtraFlag, type AppIdentifier } from "@seelen-ui/lib/types";
import { ConfigProvider, Input, Modal, Select, Switch } from "antd";
import { cloneDeep } from "lodash";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { ownSelector, RootSelectors } from "../../shared/store/app/selectors.ts";
import { defaultAppConfig } from "../app/default.ts";

import type { RootState } from "../../shared/store/domain.ts";
import type { AppConfigurationExtended } from "../domain.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { Identifier } from "./Identifier.tsx";
import cs from "./index.module.css";

interface Props {
  idx?: number;
  open: boolean;
  onSave: (app: AppConfigurationExtended) => void;
  onCancel: () => void;
  isNew?: boolean;
  readonlyApp?: AppConfigurationExtended;
}

const getAppSelector = (idx: number | undefined, isNew: boolean) =>
  createSelector([ownSelector], (state: RootState): AppConfig => {
    return idx != null && !isNew ? state.appsConfigurations[idx]! : cloneDeep(defaultAppConfig);
  });

export const EditAppModal = ({ idx, onCancel, onSave, isNew, open, readonlyApp }: Props) => {
  const { t } = useTranslation();

  const _monitors = useSelector(RootSelectors.monitorsV3);
  const monitors = Object.values(_monitors);
  const _app = useSelector(getAppSelector(idx, !!isNew));
  const initialState = readonlyApp || _app;
  const isReadonly = !!readonlyApp;

  const [app, setApp] = useState(initialState);

  useEffect(() => {
    if (isNew && !open) {
      // reset state on close
      setApp(initialState);
    }
  }, [open]);

  const onInternalSave = () => {
    onSave(app as AppConfigurationExtended);
  };

  const updateName = (e: React.ChangeEvent<HTMLInputElement>) => setApp({ ...app, name: e.currentTarget.value });
  const updateCategory = (e: React.ChangeEvent<HTMLInputElement>) =>
    setApp({ ...app, category: e.currentTarget.value || null });

  const onChangeIdentifier = (identifier: AppIdentifier) => setApp({ ...app, identifier });

  const onSelectMonitor = (value: number | null) => setApp({ ...app, boundMonitor: value });
  const onSelectWorkspace = (value: number | null) => setApp({ ...app, boundWorkspace: value });

  const onChangeOption = (option: AppExtraFlag, checked: boolean) => {
    setApp({
      ...app,
      options: checked ? [...app.options, option] : app.options.filter((o) => o !== option),
    });
  };

  const monitorsOptions = monitors.map((_, i) => ({
    label: `Monitor ${i + 1}`,
    value: i,
  }));
  const workspaceOptions = Array.from({ length: 10 }).map((_, i) => ({
    label: `Workspace ${i + 1}`,
    value: i,
  }));

  let title = t("apps_configurations.app.title_edit");
  let okText = t("apps_configurations.app.ok_edit");

  if (isNew) {
    title = t("apps_configurations.app.title_create");
    okText = t("apps_configurations.app.ok_create");
  }

  if (isReadonly) {
    title = t("apps_configurations.app.title_readonly");
    okText = t("apps_configurations.app.ok_readonly");
  }

  return (
    <Modal
      title={title.replace("{{name}}", app.name)}
      open={open}
      onCancel={onCancel}
      onOk={onInternalSave}
      cancelText={t("cancel")}
      okText={okText}
      cancelButtonProps={isReadonly ? { style: { display: "none" } } : undefined}
      centered
      className={cs.editModal}
      width="90vw"
    >
      <ConfigProvider componentDisabled={isReadonly}>
        {!!readonlyApp && (
          <SettingsGroup>
            <b>{t("apps_configurations.bundled_title")}</b>
            <p>{t("apps_configurations.bundled_msg")}</p>
          </SettingsGroup>
        )}

        <SettingsGroup>
          <SettingsOption>
            <span>{t("apps_configurations.app.name")}</span>
            <Input value={app.name} onChange={updateName} required />
          </SettingsOption>
          <SettingsOption>
            <span>{t("apps_configurations.app.category")}</span>
            <Input
              value={app.category || ""}
              placeholder={t("apps_configurations.app.category_placeholder")}
              onChange={updateCategory}
            />
          </SettingsOption>
        </SettingsGroup>

        <Identifier identifier={app.identifier} onChange={onChangeIdentifier} />

        <SettingsGroup>
          <SettingsSubGroup label={t("apps_configurations.app.bindings")}>
            <SettingsOption>
              <span>{t("apps_configurations.app.monitor")}</span>
              <Select
                value={app.boundMonitor}
                placeholder={t("apps_configurations.app.monitor_placeholder")}
                allowClear
                options={monitorsOptions}
                onChange={onSelectMonitor}
              />
            </SettingsOption>
            <SettingsOption>
              <span>{t("apps_configurations.app.workspace")}</span>
              <Select
                value={app.boundWorkspace}
                placeholder={t("apps_configurations.app.workspace_placeholder")}
                allowClear
                options={workspaceOptions}
                onChange={onSelectWorkspace}
              />
            </SettingsOption>
          </SettingsSubGroup>
        </SettingsGroup>

        <SettingsGroup>
          {Object.values(AppExtraFlag).map((value) => {
            if (value === AppExtraFlag.Unknown) return null;
            return (
              <SettingsOption
                key={value}
                label={t(`apps_configurations.app.options.${value}`)}
                action={
                  <Switch
                    value={app.options.includes(value as any as AppExtraFlag)}
                    onChange={onChangeOption.bind(this, value as any as AppExtraFlag)}
                  />
                }
              />
            );
          })}
        </SettingsGroup>
      </ConfigProvider>
    </Modal>
  );
};
