import { HideMode, SeelenWegMode, SeelenWegSide } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, InputNumber, Select, Switch } from "antd";
import { useTranslation } from "react-i18next";

import { OptionsFromEnum } from "../shared/utils/app.ts";
import { getWegConfig, patchWegConfig } from "./application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";
import Compact from "antd/es/space/Compact";

export const SeelenWegSettings = () => {
  const settings = getWegConfig();

  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    patchWegConfig({ enabled: value });
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t("weg.enable")}</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("weg.label")}>
          <SettingsOption>
            <div>{t("weg.width")}</div>
            <Select
              style={{ width: "120px" }}
              value={settings.mode}
              options={OptionsFromEnum(t, SeelenWegMode, "weg.mode")}
              onChange={(value) => patchWegConfig({ mode: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.dock_side")}</div>
            <Compact>
              {Object.values(SeelenWegSide).map((side) => (
                <Button
                  key={side}
                  type={side === settings.position ? "primary" : "default"}
                  onClick={() => patchWegConfig({ position: side })}
                >
                  <Icon iconName={`CgToolbar${side}`} size={18} />
                </Button>
              ))}
            </Compact>
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.margin")}</div>
            <InputNumber
              value={settings.margin}
              onChange={(value) => patchWegConfig({ margin: value || 0 })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.padding")}</div>
            <InputNumber
              value={settings.padding}
              onChange={(value) => patchWegConfig({ padding: value || 0 })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption>
              <b>{t("weg.auto_hide")}</b>
              <Select
                style={{ width: "120px" }}
                value={settings.hideMode}
                options={OptionsFromEnum(t, HideMode, "weg.hide_mode")}
                onChange={(value) => patchWegConfig({ hideMode: value })}
              />
            </SettingsOption>
          }
        >
          <SettingsOption>
            <span>{t("weg.delay_to_show")} (ms)</span>
            <InputNumber
              value={settings.delayToShow}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => patchWegConfig({ delayToShow: value || 0 })}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("weg.delay_to_hide")} (ms)</span>
            <InputNumber
              value={settings.delayToHide}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => patchWegConfig({ delayToHide: value || 0 })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("weg.filtering")}>
          <SettingsOption>
            <div>{t("weg.items.temporal_visibility.label")}</div>
            <Select
              style={{ width: "120px" }}
              value={settings.temporalItemsVisibility}
              options={[
                { value: "All", label: t("weg.items.temporal_visibility.all") },
                {
                  value: "OnMonitor",
                  label: t("weg.items.temporal_visibility.on_monitor"),
                },
              ]}
              onChange={(value) => patchWegConfig({ temporalItemsVisibility: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.items.pinned_visibility.label")}</div>
            <Select
              style={{ width: "120px" }}
              value={settings.pinnedItemsVisibility}
              options={[
                {
                  value: "Always",
                  label: t("weg.items.pinned_visibility.always"),
                },
                {
                  value: "WhenPrimary",
                  label: t("weg.items.pinned_visibility.when_primary"),
                },
              ]}
              onChange={(value) => patchWegConfig({ pinnedItemsVisibility: value })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("weg.items.label")}>
          <SettingsOption>
            <div>{t("weg.items.size")}</div>
            <InputNumber
              value={settings.size}
              onChange={(value) => patchWegConfig({ size: value || 0 })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.items.gap")}</div>
            <InputNumber
              value={settings.spaceBetweenItems}
              onChange={(value) => patchWegConfig({ spaceBetweenItems: value || 0 })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.items.show_window_title")}</div>
            <Switch
              checked={settings.showWindowTitle}
              onChange={(value) => patchWegConfig({ showWindowTitle: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.items.show_instance_counter")}</div>
            <Switch
              checked={settings.showInstanceCounter}
              onChange={(value) => patchWegConfig({ showInstanceCounter: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("weg.items.visible_separators")}</div>
            <Switch
              checked={settings.visibleSeparators}
              onChange={(value) => patchWegConfig({ visibleSeparators: value })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
};
