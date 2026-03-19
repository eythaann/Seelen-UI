import { SupportedLanguages } from "@seelen-ui/lib";
import { Input, InputNumber, Select, Switch } from "antd";
import { useTransition } from "react";
import { useTranslation } from "react-i18next";

import { startup } from "../../shared/tauri/infra.ts";
import { autostart } from "../../../state/system.ts";
import {
  getDateFormat,
  getHardwareAcceleration,
  getLanguage,
  getPollingInterval,
  getStartOfWeek,
  setDateFormat,
  setHardwareAcceleration,
  setLanguage,
  setPollingInterval,
  setStartOfWeek,
} from "../application.ts";

import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";
import { Colors } from "./Colors.tsx";
import { PerformanceSettings } from "./Performance.tsx";

export function General() {
  const [changingAutostart, startTransition] = useTransition();

  const language = getLanguage();
  const dateFormat = getDateFormat();
  const startOfWeek = getStartOfWeek();
  const hardwareAcceleration = getHardwareAcceleration();
  const pollingInterval = getPollingInterval();

  const { t } = useTranslation();

  function onAutoStart(value: boolean) {
    startTransition(async () => {
      try {
        if (value) {
          await startup.enable();
        } else {
          await startup.disable();
        }
        autostart.value = await startup.isEnabled();
      } catch (e) {
        console.error(e);
      }
    });
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={t("general.startup")}
          action={<Switch onChange={onAutoStart} value={autostart.value} loading={changingAutostart} />}
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("general.language")}
          action={
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
              onSelect={(value) => setLanguage(value)}
            />
          }
        />

        <SettingsOption
          label={t("general.date_format")}
          description={
            <a
              href="https://momentjs.com/docs/#/displaying/format/"
              target="_blank"
              style={{ opacity: 0.8 }}
            >
              {t("general.date_format_how_to")}
            </a>
          }
          action={
            <Input
              style={{ width: "200px", maxWidth: "200px" }}
              placeholder="YYYY-MM-DD"
              value={dateFormat}
              onChange={(e) => {
                setDateFormat(e.currentTarget.value);
              }}
            />
          }
        />

        <SettingsOption
          label={t("general.start_of_week")}
          action={
            <Select
              style={{ width: "200px" }}
              value={startOfWeek}
              options={[
                { label: t("general.monday"), value: "Monday" },
                { label: t("general.sunday"), value: "Sunday" },
                { label: t("general.saturday"), value: "Saturday" },
              ]}
              onSelect={(value) => setStartOfWeek(value)}
            />
          }
        />
      </SettingsGroup>

      <Colors />

      <SettingsGroup>
        <SettingsOption
          label={t("general.hardware_acceleration")}
          description={t("general.hardware_acceleration_description")}
          action={<Switch onChange={setHardwareAcceleration} checked={hardwareAcceleration} />}
        />

        <SettingsOption
          label={t("general.polling_interval")}
          description={t("general.polling_interval_description")}
          action={
            <InputNumber
              style={{ width: "100px" }}
              min={1}
              step={1}
              precision={0}
              value={pollingInterval}
              onChange={(value) => value !== null && setPollingInterval(value)}
            />
          }
        />
      </SettingsGroup>

      <PerformanceSettings />
    </>
  );
}
