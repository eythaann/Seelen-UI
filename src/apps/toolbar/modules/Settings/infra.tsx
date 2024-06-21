import { Icon } from '../../../shared/components/Icon';
import { SettingsToolbarModule } from '../../../shared/schemas/Placeholders';
import { invoke } from '@tauri-apps/api/core';
import { Popover, Slider, Tooltip } from 'antd';
import React, { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../seelenweg/components/BackgrounByLayers/infra';
import { Item } from '../item/infra';
import { useAppBlur } from '../shared/hooks/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: SettingsToolbarModule;
}

interface Brightness {
  min: number;
  max: number;
  current: number;
}

export function SettingsModule(props: Props) {
  const themeLayers = useSelector(Selectors.themeLayers.toolbar);
  const [openPreview, setOpenPreview] = useState(false);
  const [volume, setVolume] = useState(0);
  const [brightness, setBrightness] = useState<Brightness>({
    min: 0,
    max: 0,
    current: 0,
  });

  useEffect(() => {
    invoke<number>('get_volume_level').then((volume) => {
      setVolume(volume);
    });

    invoke<Brightness>('get_main_monitor_brightness')
      .then(setBrightness)
      .catch(() => {
        // TODO brightness is always failing
        // console.error(e);
      });
  }, [openPreview]);

  useAppBlur(() => {
    setOpenPreview(false);
  });

  const onChangeVolume = (value: number) => {
    invoke('set_volume_level', { level: value });
    setVolume(value);
  };

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={
        <div className="fast-settings">
          <BackgroundByLayers prefix="fast-settings" layers={themeLayers.fastSettings.bg} />
          <div className="fast-settings-title">
            <span>Settings</span>
            <Tooltip mouseLeaveDelay={0} arrow={false} title="App settings" placement="left">
              <button
                className="fast-settings-item-title-button"
                onClick={() => invoke('show_app_settings')}
              >
                <Icon iconName="RiSettings4Fill" />
              </button>
            </Tooltip>
          </div>
          <div className="fast-settings-item">
            <Icon iconName="IoVolumeHighOutline" />
            <Slider
              value={volume}
              onChange={onChangeVolume}
              min={0}
              max={1}
              step={0.01}
              tooltip={{
                formatter: (value) => `${(100 * (value || 0)).toFixed(0)}`,
              }}
            />
          </div>
          {brightness.max > 0 && (
            <div className="fast-settings-item">
              <Icon iconName="CiBrightnessUp" />
              <Slider
                value={brightness.current}
                onChange={(value) => setBrightness({ ...brightness, current: value })}
                min={brightness.min}
                max={brightness.max}
              />
            </div>
          )}
          <div className="fast-settings-item fast-settings-power">
            <Tooltip mouseLeaveDelay={0} arrow={false} title="Log out">
              <button className="fast-settings-item-button" onClick={() => invoke('log_out')}>
                <Icon iconName="BiLogOut" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title="Sleep">
              <button className="fast-settings-item-button" onClick={() => invoke('suspend')}>
                <Icon iconName="BiMoon" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title="Restart">
              <button className="fast-settings-item-button" onClick={() => invoke('restart')}>
                <Icon iconName="VscDebugRestart" />
              </button>
            </Tooltip>
            <Tooltip mouseLeaveDelay={0} arrow={false} title="Shut down">
              <button className="fast-settings-item-button" onClick={() => invoke('shutdown')}>
                <Icon iconName="GrPower" />
              </button>
            </Tooltip>
          </div>
        </div>
      }
    >
      <div>
        <Item
          module={{
            ...props.module,
            onClick: 'nothing',
          }}
        />
      </div>
    </Popover>
  );
}
