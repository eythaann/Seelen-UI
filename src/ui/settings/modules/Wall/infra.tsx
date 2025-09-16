import { Icon } from "@shared/components/Icon";
import { Button, InputNumber, Switch } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { Link } from "react-router";

import { newSelectors, RootActions } from "../shared/store/app/reducer";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox";
import { WallpaperList } from "./WallpaperList";
import cs from "./index.module.css";

export function WallSettings() {
  const wall = useSelector(newSelectors.wall);
  const { enabled, interval } = wall;

  const [time, setTime] = useState({
    hours: Math.floor(interval / 3600),
    minutes: Math.floor((interval / 60) % 60),
    seconds: interval % 60,
  });

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useEffect(() => {
    setTime({
      hours: Math.floor(interval / 3600),
      minutes: Math.floor((interval / 60) % 60),
      seconds: interval % 60,
    });
  }, [interval]);

  const patchWallSettings = (changes: Partial<typeof wall>) => {
    dispatch(RootActions.patchWall({ ...changes }));
  };

  function onChangeEnabled(enabled: boolean) {
    patchWallSettings({ enabled });
  }

  const updateTime = (
    key: "hours" | "minutes" | "seconds",
    value: number | null,
  ) => {
    if (value === null) return;
    const newTime = { ...time, [key]: Math.floor(value) };
    setTime(newTime);
    const newInterval = Math.max(
      newTime.hours * 3600 + newTime.minutes * 60 + newTime.seconds,
      1,
    );
    patchWallSettings({ interval: newInterval });
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("wall.enable")}</b>
          <Switch value={enabled} onChange={onChangeEnabled} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("wall.random")}</b>
          <Switch
            value={wall.randomize}
            onChange={(randomize) => patchWallSettings({ randomize })}
          />
        </SettingsOption>
        <SettingsOption>
          <b>{t("wall.interval")}</b>
          <div className={cs.interval}>
            {["hours", "minutes", "seconds"].map((unit) => (
              <div key={unit}>
                <b>{t(`wall.${unit}`)}:</b>
                <InputNumber
                  value={time[unit as keyof typeof time]}
                  onChange={(value) => updateTime(unit as "hours" | "minutes" | "seconds", value)}
                  min={0}
                  style={{ width: 50 }}
                />
              </div>
            ))}
          </div>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("wall.backgrounds")}</b>
          <Link to="/resources/wallpaper">
            <Button type="primary" className={cs.backgroundAdd}>
              <Icon iconName="IoImages" size={14} />
            </Button>
          </Link>
        </SettingsOption>
      </SettingsGroup>

      <WallpaperList />
    </>
  );
}
