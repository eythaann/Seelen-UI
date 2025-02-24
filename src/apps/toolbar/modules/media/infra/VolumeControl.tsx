import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Slider, Tooltip } from 'antd';
import { throttle } from 'lodash';
import { memo, useCallback, useEffect, useState } from 'react';

import { Icon } from 'src/apps/shared/components/Icon';

interface Props {
  value: number;
  deviceId: string;
  sessionId?: string;
}

export const VolumeSlider = memo(({ value, deviceId, sessionId }: Props) => {
  const [internalValue, setInternalValue] = useState(value);

  useEffect(() => {
    setInternalValue(value);
  }, [value]);

  const onExternalChange = useCallback(
    throttle((value: number) => {
      invoke(SeelenCommand.SetVolumeLevel, { deviceId, sessionId, level: value }).catch(console.error);
    }, 100),
    [deviceId, sessionId],
  );

  const onInternalChange = (value: number) => {
    setInternalValue(value);
    onExternalChange(value);
  };

  function onWheel(e: React.WheelEvent) {
    const isUp = e.deltaY < 0;
    const level = Math.max(0, Math.min(1, internalValue + (isUp ? 0.02 : -0.02)));
    onInternalChange(level);
  }

  return (
    <div onWheel={onWheel}>
      <Slider
        value={internalValue}
        onChange={onInternalChange}
        min={0}
        max={1}
        step={0.01}
        tooltip={{
          formatter: (value) => `${(100 * (value || 0)).toFixed(0)}%`,
        }}
      />
    </div>
  );
});

interface VolumeControlProps {
  value: number;
  icon: React.ReactNode;
  deviceId: string;
  sessionName?: string;
  sessionId?: string;
  onRightAction?: () => void;
  withPercentage?: boolean;
}

export const VolumeControl = memo((props: VolumeControlProps) => {
  const {
    value,
    icon,
    deviceId,
    sessionName,
    sessionId,
    onRightAction,
    withPercentage = false,
  } = props;

  return (
    <div className="media-control-volume">
      <Tooltip title={sessionName}>
        <button
          className="media-control-volume-action"
          onClick={() => invoke(SeelenCommand.MediaToggleMute, { deviceId, sessionId })}
        >
          {icon}
        </button>
      </Tooltip>
      <VolumeSlider value={value} deviceId={deviceId} sessionId={sessionId} />
      {withPercentage && <span style={{ lineHeight: '100%' }}>{(100 * value).toFixed(0)}%</span>}
      {!!onRightAction && (
        <button className="media-control-volume-action" onClick={onRightAction}>
          <Icon iconName="RiEqualizerLine" />
        </button>
      )}
    </div>
  );
});
