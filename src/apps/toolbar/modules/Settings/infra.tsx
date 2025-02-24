import { SeelenCommand } from '@seelen-ui/lib';
import { SettingsToolbarItem } from '@seelen-ui/lib/types';
import { invoke } from '@tauri-apps/api/core';
import { Button, Slider, Tooltip } from 'antd';
import { throttle } from 'lodash';
import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { Item } from '../item/infra/infra';
import { VolumeControl } from '../media/infra/VolumeControl';

import { Selectors } from '../shared/store/app';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { RootState } from '../shared/store/domain';

import { AnimatedPopover } from '../../../shared/components/AnimatedWrappers';
import { Icon } from '../../../shared/components/Icon';

interface Props {
  module: SettingsToolbarItem;
}

interface Brightness {
  min: number;
  max: number;
  current: number;
}

function brightnessIcon(brightness: number) {
  if (brightness >= 60) {
    return 'TbBrightnessUp';
  }
  return brightness >= 30 ? 'TbBrightnessDown' : 'TbBrightnessDownFilled';
}

export function SettingsModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);
  const [brightness, setBrightness] = useState<Brightness | null>(null);

  const showHibernate = useSelector(Selectors.settings.showHibernateButton);
  const defaultInput = useSelector((state: RootState) =>
    Selectors.mediaInputs(state).find((d) => d.isDefaultMultimedia),
  );
  const defaultOutput = useSelector((state: RootState) =>
    Selectors.mediaOutputs(state).find((d) => d.isDefaultMultimedia),
  );

  const { t } = useTranslation();

  useEffect(() => {
    invoke<Brightness | null>('get_main_monitor_brightness').then(setBrightness);
  }, [openPreview]);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  const setBrightnessExternal = useCallback(
    throttle((brightness: number) => {
      invoke('set_main_monitor_brightness', { brightness });
    }, 100),
    [],
  );

  return (
    <AnimatedPopover
      animationDescription={{
        openAnimationName: 'settings-open',
        closeAnimationName: 'settings-close',
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <div className="fast-settings" onContextMenu={(e) => e.stopPropagation()}>
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

          {brightness && <span className="fast-settings-label">{t('settings.brightness')}</span>}
          {brightness && (
            <div className="fast-settings-item">
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
                // onRightAction={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:sound' })}
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

          <span className="fast-settings-label">{t('settings.power')}</span>
          <div className="fast-settings-item fast-settings-power">
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.lock')}>
              <button
                className="fast-settings-item-button"
                onClick={() => invoke(SeelenCommand.Lock)}
              >
                <Icon iconName="BiLock" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.sleep')}>
              <button
                className="fast-settings-item-button"
                onClick={() => invoke(SeelenCommand.Suspend)}
              >
                <Icon iconName="BiMoon" />
              </button>
            </Tooltip>
            {showHibernate && (
              <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.hibernate')}>
                <button
                  className="fast-settings-item-button"
                  onClick={() => invoke(SeelenCommand.Hibernate)}
                >
                  <Icon iconName="FiClock" />
                </button>
              </Tooltip>
            )}
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.restart')}>
              <button
                className="fast-settings-item-button"
                onClick={() => invoke(SeelenCommand.Restart)}
              >
                <Icon iconName="VscDebugRestart" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.shutdown')}>
              <button
                className="fast-settings-item-button"
                onClick={() => invoke(SeelenCommand.Shutdown)}
              >
                <Icon iconName="GrPower" />
              </button>
            </Tooltip>
          </div>
        </div>
      }
    >
      <Item module={module} active={openPreview} />
    </AnimatedPopover>
  );
}
