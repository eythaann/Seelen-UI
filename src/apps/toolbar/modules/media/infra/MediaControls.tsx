import { SeelenCommand } from '@seelen-ui/lib';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Button, Popover, Slider, Tooltip } from 'antd';
import { debounce } from 'lodash';
import React, { memo, PropsWithChildren, useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { selectDefaultOutput, Selectors } from '../../shared/store/app';
import { calcLuminance } from '../application';

import { MediaChannelTransportData, MediaDevice } from '../../shared/store/domain';

import AnimatedPopover from '../../../../shared/components/AnimatedPopover';
import { Icon } from '../../../../shared/components/Icon';
import { OverflowTooltip } from '../../../../shared/components/OverflowTooltip';
import { useTimeout, useWindowFocusChange } from '../../../../shared/hooks';

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
      <div className="media-session-thumbnail-container">
        {session.owner && (
          <Tooltip title={session.owner.name} placement="bottom">
            <img
              className="media-session-app-icon"
              src={convertFileSrc(
                session.owner.iconPath ? session.owner.iconPath : LAZY_CONSTANTS.MISSING_ICON_PATH,
              )}
              draggable={false}
            />
          </Tooltip>
        )}
        <img className="media-session-thumbnail" src={src} draggable={false} />
      </div>
      <img className="media-session-blurred-thumbnail" src={src} draggable={false} />

      <div className="media-session-info" style={{ color }}>
        <h4 className="media-session-title">{session.title}</h4>
        <span className="media-session-author">{session.author}</span>
        <div className="media-session-actions">
          <Button type="text" onClick={onClickBtn.bind(null, 'media_prev')}>
            <Icon iconName="TbPlayerSkipBackFilled" color={color} />
          </Button>
          <Button type="text" onClick={onClickBtn.bind(null, 'media_toggle_play_pause')}>
            <Icon
              iconName={session.playing ? 'TbPlayerPauseFilled' : 'TbPlayerPlayFilled'}
              color={color}
            />
          </Button>
          <Button type="text" onClick={onClickBtn.bind(null, 'media_next')}>
            <Icon iconName="TbPlayerSkipForwardFilled" color={color} />
          </Button>
        </div>
      </div>
    </div>
  );
}

function Device({ device }: { device: MediaDevice }) {
  const { t } = useTranslation();

  const onClickMultimedia = () => {
    if (!device.is_default_multimedia) {
      invoke(SeelenCommand.MediaSetDefaultDevice, { id: device.id, role: 'multimedia' })
        .then(() => invoke(SeelenCommand.MediaSetDefaultDevice, { id: device.id, role: 'console' }))
        .catch(console.error);
    }
  };

  const onClickCommunications = () => {
    if (!device.is_default_communications) {
      invoke(SeelenCommand.MediaSetDefaultDevice, { id: device.id, role: 'communications' }).catch(
        console.error,
      );
    }
  };

  return (
    <div className="media-device">
      <Button.Group size="small" style={{ width: 50 }}>
        <Tooltip title={t('media.device.multimedia')}>
          <Button
            type={device.is_default_multimedia ? 'primary' : 'default'}
            onClick={onClickMultimedia}
          >
            <Icon iconName="IoMusicalNotes" size={18} />
          </Button>
        </Tooltip>
        <Tooltip title={t('media.device.comunications')}>
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
  withRightAction?: boolean;
  withPercentage?: boolean;
}

const tooltipVisibilityTimeout = 3 * 1000;

export const VolumeControl = memo((props: VolumeControlProps) => {
  const { value, icon, deviceId, sessionId, withRightAction = true, withPercentage = false } = props;

  const [internalValue, setInternalValue] = useState(value);
  const [openTooltip, setOpenTooltip] = useState(false);

  useEffect(() => {
    setInternalValue(value);
  }, [value]);

  const onExternalChange = useCallback(
    debounce((value: number) => {
      invoke(SeelenCommand.SetVolumeLevel, { id: deviceId, level: value }).catch(console.error);
    }, 100),
    [deviceId, sessionId],
  );

  useTimeout(() => {
    setOpenTooltip(false);
  },
  tooltipVisibilityTimeout,
  [openTooltip]);

  const onInternalChange = (value: number) => {
    setInternalValue(value);
    setOpenTooltip(!withPercentage);
    onExternalChange(value);
  };

  function onWheel(e: React.WheelEvent) {
    const isUp = e.deltaY < 0;
    const level = Math.max(0, Math.min(1, internalValue + (isUp ? 0.02 : -0.02)));
    onInternalChange(level);
  }

  return (
    <div className="media-control-volume" onWheel={onWheel}>
      <Button type="text" onClick={() => invoke(SeelenCommand.MediaToggleMute, { id: deviceId })}>
        {icon}
      </Button>
      <Slider
        value={internalValue}
        onChange={onInternalChange}
        min={0}
        max={1}
        step={0.01}
        tooltip={{
          open: openTooltip,
          onOpenChange: (open) => setOpenTooltip(!withPercentage && open),
          formatter: (value) => `${(100 * (value || 0)).toFixed(0)}%`,
        }}
      />
      {withPercentage && <span style={{ lineHeight: '100%' }}>{Math.round(internalValue * 100)}%</span>}
      {withRightAction && (
        <Button
          type="text"
          onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:sound' })}
        >
          <Icon iconName="RiEqualizerLine" />
        </Button>
      )}
    </div>
  );
});

function MediaControls() {
  const { t } = useTranslation();

  const inputs = useSelector(Selectors.mediaInputs);
  const defaultInput = inputs.find((d) => d.is_default_multimedia);

  const outputs = useSelector(Selectors.mediaOutputs);
  const defaultOutput = outputs.find((d) => d.is_default_multimedia);

  const sessions = useSelector(Selectors.mediaSessions);

  return (
    <BackgroundByLayersV2 className="media-control" onContextMenu={(e) => e.stopPropagation()}>
      <span className="media-control-label">{t('media.master_volume')}</span>
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
          <span className="media-control-label">{t('media.output_device')}</span>
          <DeviceGroup devices={outputs} />
        </>
      )}

      {inputs.length > 0 && (
        <>
          <span className="media-control-label">{t('media.input_device')}</span>
          <DeviceGroup devices={inputs} />
        </>
      )}

      {sessions.length > 0 && (
        <>
          <span className="media-control-label">{t('media.players')}</span>
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
  const [openControls, setOpenControls] = useState(false);
  const [openNotifier, setOpenNotifier] = useState(false);

  const defaultOutput = useSelector(selectDefaultOutput);

  const firstLoad = useRef(true);

  const closeVolumeNotifier = useCallback(
    debounce(() => setOpenNotifier(false), 2000),
    [],
  );

  useEffect(() => {
    if (!defaultOutput) {
      return;
    }

    if (firstLoad.current) {
      firstLoad.current = false;
      return;
    }

    if (!openControls && !openNotifier) {
      setOpenNotifier(true);
    }
    closeVolumeNotifier();
  }, [defaultOutput?.volume]);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenControls(false);
    }
  });

  return (
    <AnimatedPopover
      animationDescription={{
        maxAnimationTimeMs: 500,
        openAnimationName: 'media-open',
        closeAnimationName: 'media-close',
      }}
      open={openControls}
      trigger="click"
      onOpenChange={(open) => {
        setOpenControls(open);
        if (open) {
          setOpenNotifier(false);
        }
      }}
      arrow={false}
      content={<MediaControls />}
      destroyTooltipOnHide
    >
      <Popover
        open={openNotifier}
        arrow={false}
        trigger={[]}
        destroyTooltipOnHide
        content={
          <BackgroundByLayersV2 className="media-notifier" onContextMenu={(e) => e.stopPropagation()}>
            {defaultOutput && (
              <VolumeControl
                value={defaultOutput.volume}
                deviceId={defaultOutput.id}
                icon={
                  <Icon
                    iconName={defaultOutput.muted ? 'IoVolumeMuteOutline' : 'IoVolumeHighOutline'}
                  />
                }
                withRightAction={false}
                withPercentage={true}
              />
            )}
          </BackgroundByLayersV2>
        }
      >
        {children}
      </Popover>
    </AnimatedPopover>
  );
}
