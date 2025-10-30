import { invoke, SeelenCommand, SeelenEvent } from "@seelen-ui/lib";
import type { Brightness, SettingsToolbarItem } from "@seelen-ui/lib/types";
import { AnimatedPopover } from "@shared/components/AnimatedWrappers";
import { Icon } from "@shared/components/Icon";
import { useWindowFocusChange } from "@shared/hooks";
import { Button, Slider, Tooltip } from "antd";
import { throttle } from "lodash";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { Item } from "../item/infra/infra.tsx";
import { VolumeControl } from "../media/infra/VolumeControl.tsx";
import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { Selectors } from "../shared/store/app.ts";

import type { RootState } from "../shared/store/domain.ts";

import { emit } from "@tauri-apps/api/event";

interface Props {
  module: SettingsToolbarItem;
}

function brightnessIcon(brightness: number) {
  if (brightness >= 60) {
    return "TbBrightnessUp";
  }
  return brightness >= 30 ? "TbBrightnessDown" : "TbBrightnessDownFilled";
}

export function SettingsModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);
  const [brightness, setBrightness] = useState<Brightness | null>(null);

  const defaultInput = useSelector((state: RootState) =>
    Selectors.mediaInputs(state).find((d) => d.isDefaultMultimedia)
  );
  const defaultOutput = useSelector((state: RootState) =>
    Selectors.mediaOutputs(state).find((d) => d.isDefaultMultimedia)
  );

  const { t } = useTranslation();

  useEffect(() => {
    invoke(SeelenCommand.GetMainMonitorBrightness).then(setBrightness);
  }, [openPreview]);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  const setBrightnessExternal = useCallback(
    throttle((brightness: number) => {
      invoke(SeelenCommand.SetMainMonitorBrightness, { brightness });
    }, 100),
    [],
  );

  return (
    <AnimatedPopover
      animationDescription={{
        openAnimationName: "settings-open",
        closeAnimationName: "settings-close",
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      content={
        <BackgroundByLayersV2
          className="quick-access"
          prefix="quick-access"
          onContextMenu={(e) => e.stopPropagation()}
        >
          <div className="quick-access-header">
            <span>{t("settings.title")}</span>
          </div>

          {brightness && <span className="quick-access-label">{t("settings.brightness")}</span>}
          {brightness && (
            <div className="quick-access-item">
              <Button
                type="text"
                onClick={() => {
                  /* TODO: add auto brightness toggle */
                }}
              >
                <Icon size={20} iconName={brightnessIcon(brightness.current)} />
              </Button>
              <Slider
                value={brightness.current}
                onChange={(current) => {
                  setBrightness({ ...brightness, current });
                  setBrightnessExternal(current);
                }}
                min={brightness.min}
                max={brightness.max}
              />
            </div>
          )}

          {!!(defaultInput || defaultOutput) && (
            <span className="quick-access-label">{t("media.default_multimedia_volume")}</span>
          )}
          {!!defaultOutput && (
            <div className="quick-access-item">
              <VolumeControl
                value={defaultOutput.volume}
                deviceId={defaultOutput.id}
                icon={
                  <Icon
                    iconName={defaultOutput.muted ? "IoVolumeMuteOutline" : "IoVolumeHighOutline"}
                  />
                }
                // onRightAction={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:sound' })}
              />
            </div>
          )}
          {!!defaultInput && (
            <div className="quick-access-item">
              <VolumeControl
                value={defaultInput.volume}
                deviceId={defaultInput.id}
                icon={<Icon iconName={defaultInput.muted ? "BiMicrophoneOff" : "BiMicrophone"} />}
              />
            </div>
          )}

          <span className="quick-access-label"></span>
          <div className="quick-access-footer">
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t("settings.app_settings")}
              placement="left"
            >
              <button
                className="quick-access-footer-button"
                onClick={() => invoke(SeelenCommand.ShowAppSettings)}
              >
                <Icon iconName="RiSettings4Fill" />
              </button>
            </Tooltip>

            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t("settings.power")}
              placement="bottom"
            >
              <button
                className="quick-access-footer-button"
                onClick={() => {
                  emit(SeelenEvent.WidgetTriggered, { id: "@seelen/power-menu" });
                }}
              >
                <Icon iconName="IoPower" />
              </button>
            </Tooltip>
          </div>
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} active={openPreview} />
    </AnimatedPopover>
  );
}
