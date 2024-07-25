import { Icon } from '../../../../shared/components/Icon';
import { OverflowTooltip } from '../../../../shared/components/OverflowTooltip';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Button, Popover, Slider, Tooltip } from 'antd';
import { debounce } from 'lodash';
import React, { PropsWithChildren, useCallback, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgrounByLayers/infra';
import { useAppBlur } from '../../shared/hooks/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import { calcLuminance } from '../application';

import { MediaChannelTransportData, MediaDevice } from '../../shared/store/domain';

import './index.css';

const MAX_LUMINANCE = 210;
const MIN_LUMINANCE = 40;
const BRIGHTNESS_MULTIPLIER = 1.5; // used in css

function MediaSession({ session }: { session: MediaChannelTransportData }) {
  const [luminance, setLuminance] = useState(0);

  let src = convertFileSrc(
    session.thumbnail ? session.thumbnail : LAZY_CONSTANTS.DEFAULT_THUMBNAIL,
  );

  useEffect(() => {
    calcLuminance(src).then(setLuminance).catch(console.error);
  }, [src]);

  const filteredLuminance = Math.max(
    Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE),
    MIN_LUMINANCE,
  );
  const color = filteredLuminance < 125 ? '#efefef' : '#222222';

  const onClickBtn = (cmd: string) => {
    invoke(cmd, { id: session.id }).catch(console.error);
  };

  return (
    <div
      className="media-session"
      style={{
        backgroundColor: `rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`,
      }}
    >
      <img className="media-session-thumbnail" src={src} draggable={false} />
      <img className="media-session-blurred-thumbnail" src={src} draggable={false} />

      <div className="media-session-info" style={{ color }}>
        <h4 className="media-session-title">{session.title}</h4>
        <span className="media-session-author">{session.author}</span>
        <div className="media-session-actions">
          <Button type="text" onClick={onClickBtn.bind(null, 'media_prev')}>
            <Icon iconName="TbPlayerSkipBackFilled" propsIcon={{ color }} />
          </Button>
          <Button type="text" onClick={onClickBtn.bind(null, 'media_toggle_play_pause')}>
            <Icon
              iconName={session.playing ? 'TbPlayerPauseFilled' : 'TbPlayerPlayFilled'}
              propsIcon={{ color }}
            />
          </Button>
          <Button type="text" onClick={onClickBtn.bind(null, 'media_next')}>
            <Icon iconName="TbPlayerSkipForwardFilled" propsIcon={{ color }} />
          </Button>
        </div>
      </div>
    </div>
  );
}

function Device({ device }: { device: MediaDevice }) {
  const onClickMultimedia = () => {
    if (!device.is_default_multimedia) {
      invoke('media_set_default_device', { id: device.id, role: 'multimedia' })
        .then(() => invoke('media_set_default_device', { id: device.id, role: 'console' }))
        .catch(console.error);
    }
  };

  const onClickCommunications = () => {
    if (!device.is_default_communications) {
      invoke('media_set_default_device', { id: device.id, role: 'communications' }).catch(
        console.error,
      );
    }
  };

  return (
    <div className="media-device">
      <Button.Group size="small" style={{ width: 50 }}>
        <Tooltip title="Multimedia">
          <Button
            type={device.is_default_multimedia ? 'primary' : 'default'}
            onClick={onClickMultimedia}
          >
            <Icon iconName="IoMusicalNotes" propsIcon={{ size: 18 }} />
          </Button>
        </Tooltip>
        <Tooltip title="Communications">
          <Button
            type={device.is_default_communications ? 'primary' : 'default'}
            onClick={onClickCommunications}
          >
            <Icon iconName="FaPhoneFlip" />
          </Button>
        </Tooltip>
      </Button.Group>
      <OverflowTooltip text={device.name} />
    </div>
  );
}

function DeviceGroup({ devices }: { devices: MediaDevice[] }) {
  return (
    <div className="media-device-group">
      {devices.map((d) => (
        <Device key={d.id} device={d} />
      ))}
    </div>
  );
}

interface VolumeControlProps {
  value: number;
  icon: React.ReactNode;
  deviceId: string;
  sessionId?: string;
}

function VolumeControl(props: VolumeControlProps) {
  const { value, icon, deviceId, sessionId } = props;

  const [internalValue, setInternalValue] = useState(value);

  useEffect(() => {
    setInternalValue(value);
  }, [value]);

  const onExternalChange = useCallback(
    debounce((value: number) => {
      invoke('set_volume_level', { id: deviceId, level: value }).catch(console.error);
    }, 100),
    [deviceId, sessionId],
  );

  const onInternalChange = (value: number) => {
    setInternalValue(value);
    onExternalChange(value);
  };

  return (
    <div className="media-control-volume">
      <Button type="text" onClick={() => invoke('media_toggle_mute', { id: deviceId })}>
        {icon}
      </Button>
      <Slider
        value={internalValue}
        onChange={onInternalChange}
        min={0}
        max={1}
        step={0.01}
        tooltip={{
          formatter: (value) => `${(100 * (value || 0)).toFixed(0)}`,
        }}
      />
      <Button type="text" onClick={() => invoke('open_file', { path: 'ms-settings:sound' })}>
        <Icon iconName="RiEqualizerLine" />
      </Button>
    </div>
  );
}

function MediaControls() {
  const inputs = useSelector(Selectors.mediaInputs);
  const defaultInput = inputs.find((d) => d.is_default_multimedia);

  const outputs = useSelector(Selectors.mediaOutputs);
  const defaultOutput = outputs.find((d) => d.is_default_multimedia);

  const sessions = useSelector(Selectors.mediaSessions);

  return (
    <BackgroundByLayersV2 className="media-control" amount={1}>
      <span className="media-control-label">Master Volume</span>
      {!!defaultOutput && (
        <VolumeControl
          value={defaultOutput.volume}
          deviceId={defaultOutput.id}
          icon={
            <Icon iconName={defaultOutput.muted ? 'IoVolumeMuteOutline' : 'IoVolumeHighOutline'} />
          }
        />
      )}

      {!!defaultInput && (
        <VolumeControl
          value={defaultInput.volume}
          deviceId={defaultInput.id}
          icon={<Icon iconName={defaultInput.muted ? 'BiMicrophoneOff' : 'BiMicrophone'} />}
        />
      )}

      {outputs.length > 0 && (
        <>
          <span className="media-control-label">Output Device</span>
          <DeviceGroup devices={outputs} />
        </>
      )}

      {inputs.length > 0 && (
        <>
          <span className="media-control-label">Input Device</span>
          <DeviceGroup devices={inputs} />
        </>
      )}

      {sessions.length > 0 && (
        <>
          <span className="media-control-label">Media Players</span>
          <div className="media-control-session-list">
            {sessions.map((session, index) => (
              <MediaSession key={index} session={session} />
            ))}
          </div>
        </>
      )}
    </BackgroundByLayersV2>
  );
}

export function WithMediaControls({ children }: PropsWithChildren) {
  const [openPreview, setOpenPreview] = useState(false);

  useAppBlur(() => {
    setOpenPreview(false);
  });

  return (
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<MediaControls />}
    >
      {children}
    </Popover>
  );
}
