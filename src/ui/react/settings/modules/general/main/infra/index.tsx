import { SupportedLanguages } from "@seelen-ui/lib";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Input, Select, Switch, Tooltip } from "antd";
import { type ChangeEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { startup } from "../../../shared/tauri/infra.ts";
import { useAppDispatch } from "../../../shared/utils/infra.ts";

import { RootActions } from "../../../shared/store/app/reducer.ts";
import { RootSelectors } from "../../../shared/store/app/selectors.ts";

import { SettingsGroup, SettingsOption } from "../../../../components/SettingsBox/index.tsx";
import { Colors } from "./Colors.tsx";
import { PerformanceSettings } from "./Performance.tsx";

export function General() {
  const [changingAutostart, setChangingAutostart] = useState(false);

  const autostartStatus = useSelector(RootSelectors.autostart);
  const language = useSelector(RootSelectors.language);
  const dateFormat = useSelector(RootSelectors.dateFormat);

  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const onAutoStart = async (value: boolean) => {
    setChangingAutostart(true);
    try {
      if (value) {
        await startup.enable();
      } else {
        await startup.disable();
      }
      dispatch(RootActions.setAutostart(await startup.isEnabled()));
    } catch (e) {
      console.error(e);
    }
    setChangingAutostart(false);
  };

  const onDateFormatChange = (e: ChangeEvent<HTMLInputElement>) =>
    dispatch(RootActions.setDateFormat(e.currentTarget.value));

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("general.startup")}</b>
          <Switch
            onChange={onAutoStart}
            value={!!autostartStatus}
            loading={changingAutostart || autostartStatus === null}
          />
        </SettingsOption>
      </SettingsGroup>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("general.language")}</b>
          <Select
            showSearch
            filterOption={(_searching, option) => {
              if (!option) {
                return true;
              }
              const searching = _searching.toLocaleLowerCase();
              let label = option.label.toLocaleLowerCase();
              let enLabel = option.enLabel.toLocaleLowerCase();
              return label.includes(searching) || enLabel.includes(searching);
            }}
            style={{ width: "200px" }}
            value={language}
            options={[...SupportedLanguages]}
            onSelect={(value) => dispatch(RootActions.setLanguage(value))}
          />
        </SettingsOption>
        <SettingsOption>
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <b>{t("general.date_format")}</b>
            <Tooltip
              title={
                <a
                  href="https://momentjs.com/docs/#/displaying/format/"
                  target="_blank"
                >
                  https://momentjs.com/docs/#/displaying/format/
                </a>
              }
            >
              <Icon iconName="LuCircleHelp" />
            </Tooltip>
          </div>
          <Input
            style={{ width: "200px", maxWidth: "200px" }}
            placeholder="YYYY-MM-DD"
            value={dateFormat}
            onChange={onDateFormatChange}
          />
        </SettingsOption>
      </SettingsGroup>

      <Colors />

      <PerformanceSettings />
    </>
  );
}
