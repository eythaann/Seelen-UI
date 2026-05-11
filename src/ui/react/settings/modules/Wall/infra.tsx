import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, InputNumber, Select, Switch } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import {
  getWallConfig,
  getWallpaperCollections,
  patchWallConfig,
  setDefaultWallpaperCollection,
} from "./application.ts";

import { MultimonitorBehaviour } from "@seelen-ui/lib/types";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";

export function WallSettings() {
  const wall = getWallConfig();
  const wallpaperCollections = getWallpaperCollections();
  const { enabled, interval } = wall;

  const [time, setTime] = useState({
    hours: Math.floor(interval / 3600),
    minutes: Math.floor((interval / 60) % 60),
  });

  const { t } = useTranslation();

  useEffect(() => {
    setTime({
      hours: Math.floor(interval / 3600),
      minutes: Math.floor((interval / 60) % 60),
    });
  }, [interval]);

  function onChangeEnabled(enabled: boolean) {
    patchWallConfig({ enabled });
  }

  const updateTime = (key: "hours" | "minutes", value: number | null) => {
    if (value === null) return;
    const newTime = { ...time, [key]: Math.floor(value) };
    setTime(newTime);
    const newInterval = Math.max(newTime.hours * 3600 + newTime.minutes * 60, 60);
    patchWallConfig({ interval: newInterval });
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={<b>{t("wall.enable")}</b>}
          action={<Switch value={enabled} onChange={onChangeEnabled} />}
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("wall.wallpapers_and_collections")}
          action={
            <Link to="/resources/wallpaper">
              <Button type="primary">
                <Icon iconName="IoImages" />
              </Button>
            </Link>
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={<b>{t("wall.use_accent_color")}</b>}
          action={
            <Switch
              value={wall.useAccentColor}
              onChange={(useAccentColor) => patchWallConfig({ useAccentColor })}
            />
          }
        />
        <SettingsOption
          label={<b>{t("wall.coverage_pause_threshold")}</b>}
          action={
            <InputNumber
              value={wall.coveragePauseThreshold}
              formatter={(v) => `${(v || 0) * 100}%`}
              parser={(v) => Number(v?.replace("%", "") || 0) / 100}
              onChange={(value) => {
                if (value === null) return;
                patchWallConfig({ coveragePauseThreshold: Math.min(1.0, Math.max(0.5, value)) });
              }}
              min={0.5}
              max={1.0}
              step={0.05}
              precision={2}
              style={{ width: 80 }}
            />
          }
        />
        <SettingsOption
          label={<b>{t("wall.multimonitor_behaviour")}</b>}
          action={
            <Select
              style={{ width: 200 }}
              value={wall.multimonitorBehaviour}
              onChange={(value) => patchWallConfig({ multimonitorBehaviour: value })}
              options={[
                {
                  label: t("wall.per_monitor"),
                  value: MultimonitorBehaviour.PerMonitor,
                },
                {
                  label: t("wall.extend"),
                  value: MultimonitorBehaviour.Extend,
                },
              ]}
            />
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={<b>{t("wall.random")}</b>}
          action={
            <Switch
              value={wall.randomize}
              onChange={(randomize) => patchWallConfig({ randomize })}
            />
          }
        />

        <SettingsSubGroup label={t("wall.interval")}>
          <SettingsOption
            label={t("wall.hours")}
            action={
              <InputNumber
                value={time.hours}
                onChange={(value) => updateTime("hours", value)}
                min={0}
                style={{ width: 50 }}
              />
            }
          />
          <SettingsOption
            label={t("wall.minutes")}
            action={
              <InputNumber
                value={time.minutes}
                onChange={(value) => updateTime("minutes", value)}
                min={0}
                style={{ width: 50 }}
              />
            }
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("wall.default_collection")}
          action={
            <Select
              style={{ width: 200 }}
              value={wall.defaultCollection ?? undefined}
              onChange={(value) => setDefaultWallpaperCollection(value || null)}
              placeholder={t("wall.select_collection")}
              allowClear
            >
              {wallpaperCollections.map((collection) => (
                <Select.Option key={collection.id} value={collection.id}>
                  {collection.name}
                </Select.Option>
              ))}
            </Select>
          }
        />
      </SettingsGroup>
    </>
  );
}
