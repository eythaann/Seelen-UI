import { PerformanceMode, type PerformanceModeSettings } from "@seelen-ui/lib/types";
import { Select } from "antd";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../../components/SettingsBox/index.tsx";

import { newSelectors, RootActions } from "../../../shared/store/app/reducer.ts";

export function PerformanceSettings() {
  const perf = useSelector(newSelectors.performanceMode);

  const { t } = useTranslation();
  const d = useDispatch();

  function patchPerfSettings(patch: Partial<PerformanceModeSettings>) {
    d(RootActions.setPerformanceMode({ ...perf, ...patch }));
  }

  const options: { label: string; value: PerformanceMode }[] = [
    {
      label: t("general.performance_mode.options.disabled"),
      value: PerformanceMode.Disabled,
    },
    {
      label: t("general.performance_mode.options.minimal"),
      value: PerformanceMode.Extreme,
    },
    {
      label: t("general.performance_mode.options.extreme"),
      value: PerformanceMode.Minimal,
    },
  ];

  return (
    <SettingsGroup>
      <SettingsSubGroup label="Performance Mode">
        <SettingsOption
          label={t("general.performance_mode.plugged")}
          action={
            <Select
              value={perf.default}
              options={options}
              onSelect={(value) => patchPerfSettings({ default: value })}
            />
          }
        />
        <SettingsOption
          label={t("general.performance_mode.on_battery")}
          action={
            <Select
              value={perf.onBattery}
              options={options}
              onSelect={(onBattery) => patchPerfSettings({ onBattery })}
            />
          }
        />
        <SettingsOption
          label={t("general.performance_mode.on_energy_saver")}
          action={
            <Select
              value={perf.onEnergySaver}
              options={options}
              onSelect={(onEnergySaver) => patchPerfSettings({ onEnergySaver })}
            />
          }
        />
      </SettingsSubGroup>
    </SettingsGroup>
  );
}
