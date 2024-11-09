import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Popover, Tooltip } from 'antd';
import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { SeelenCommand, useWindowFocusChange } from 'seelen-core';
import { SettingsToolbarModule } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra/infra';
import { VolumeControl } from '../media/infra/MediaControls';

import { Selectors } from '../shared/store/app';

import { RootState } from '../shared/store/domain';

import { Icon } from '../../../shared/components/Icon';

interface Props {
  module: SettingsToolbarModule;
}

interface Brightness {
  min: number;
  max: number;
  current: number;
}

export function SettingsModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);
  const [_brightness, setBrightness] = useState<Brightness>({
    min: 0,
    max: 0,
    current: 0,
  });

  const defaultInput = useSelector((state: RootState) =>
    Selectors.mediaInputs(state).find((d) => d.is_default_multimedia),
  );
  const defaultOutput = useSelector((state: RootState) =>
    Selectors.mediaOutputs(state).find((d) => d.is_default_multimedia),
  );

  const { t } = useTranslation();

  useEffect(() => {
    emit('register-media-events');
  }, []);

  useEffect(() => {
    invoke<Brightness>('get_main_monitor_brightness')
      .then(setBrightness)
      .catch(() => {
        // TODO brightness is always failing
        // console.error(e);
      });
  }, [openPreview]);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <div className="fast-settings">
          <BackgroundByLayersV2 prefix="fast-settings" />
          <div className="fast-settings-title">
            <span>{t('settings.title')}</span>
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t('settings.app_settings')}
              placement="left"
            >
              <button
                className="fast-settings-item-title-button"
                onClick={() => invoke(SeelenCommand.ShowAppSettings)}
              >
                <Icon iconName="RiSettings4Fill" />
              </button>
            </Tooltip>
          </div>

          {!!(defaultInput || defaultOutput) && (
            <span className="fast-settings-label">{t('media.master_volume')}</span>
          )}

          {!!defaultOutput && (
            <div className="fast-settings-item">
              <VolumeControl
                value={defaultOutput.volume}
                deviceId={defaultOutput.id}
                icon={
                  <Icon
                    iconName={defaultOutput.muted ? 'IoVolumeMuteOutline' : 'IoVolumeHighOutline'}
                  />
                }
              />
            </div>
          )}

          {!!defaultInput && (
            <div className="fast-settings-item">
              <VolumeControl
                value={defaultInput.volume}
                deviceId={defaultInput.id}
                icon={<Icon iconName={defaultInput.muted ? 'BiMicrophoneOff' : 'BiMicrophone'} />}
              />
            </div>
          )}

          {/* brightness.max > 0 && (
            <div className="fast-settings-item">
              <Icon iconName="CiBrightnessUp" />
              <Slider
                value={brightness.current}
                onChange={(value) => setBrightness({ ...brightness, current: value })}
                min={brightness.min}
                max={brightness.max}
              />
            </div>
          ) */}

          <span className="fast-settings-label">{t('settings.power')}</span>
          <div className="fast-settings-item fast-settings-power">
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.log_out')}>
              <button className="fast-settings-item-button" onClick={() => invoke(SeelenCommand.LogOut)}>
                <Icon iconName="BiLogOut" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.sleep')}>
              <button className="fast-settings-item-button" onClick={() => invoke(SeelenCommand.Suspend)}>
                <Icon iconName="BiMoon" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.restart')}>
              <button className="fast-settings-item-button" onClick={() => invoke(SeelenCommand.Restart)}>
                <Icon iconName="VscDebugRestart" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.shutdown')}>
              <button className="fast-settings-item-button" onClick={() => invoke(SeelenCommand.Shutdown)}>
                <Icon iconName="GrPower" />
              </button>
            </Tooltip>
          </div>
        </div>
      }
    >
      <Item module={module} />
    </Popover>
  );
}
