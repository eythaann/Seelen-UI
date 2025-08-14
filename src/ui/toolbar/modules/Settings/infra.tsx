import { useComputed } from '@preact/signals';
import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { Brightness, SettingsToolbarItem } from '@seelen-ui/lib/types';
import { AnimatedPopover } from '@shared/components/AnimatedWrappers';
import { Icon } from '@shared/components/Icon';
import { useWindowFocusChange } from '@shared/hooks';
import { Button, Slider, Tooltip } from 'antd';
import { throttle } from 'lodash';
import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Item } from '../item/infra/infra';
import { VolumeControl } from '../media/infra/VolumeControl';
import { BackgroundByLayersV2 } from '@shared/components/BackgroundByLayers/infra';

import { Selectors } from '../shared/store/app';

import { RootState } from '../shared/store/domain';

import { $settings } from '../shared/state/mod';

interface Props {
  module: SettingsToolbarItem;
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

  const showHibernate = useComputed(() => $settings.value.showHibernateButton);
  const defaultInput = useSelector((state: RootState) =>
    Selectors.mediaInputs(state).find((d) => d.isDefaultMultimedia),
  );
  const defaultOutput = useSelector((state: RootState) =>
    Selectors.mediaOutputs(state).find((d) => d.isDefaultMultimedia),
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
        openAnimationName: 'settings-open',
        closeAnimationName: 'settings-close',
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      content={
        <BackgroundByLayersV2
          className="fast-settings"
          prefix="fast-settings"
          onContextMenu={(e) => e.stopPropagation()}
        >
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
            <span className="fast-settings-label">{t('media.default_multimedia_volume')}</span>
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
        </BackgroundByLayersV2>
      }
    >
      <Item module={module} active={openPreview} />
    </AnimatedPopover>
  );
}
