import {
  SUPPORTED_IMAGE_WALLPAPER_EXTENSIONS,
  SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS,
} from '@seelen-ui/lib';
import { Wallpaper, WallpaperInstanceSettings } from '@seelen-ui/lib/types';
import { cx } from '@shared/styles';
import { convertFileSrc } from '@tauri-apps/api/core';
import { ComponentChildren } from 'preact';
import { useEffect, useRef } from 'preact/hooks';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { getPlaybackRate, getWallpaperStyles } from './utils';
import cs from './index.module.css';

interface Props {
  definition: Wallpaper;
  config: WallpaperInstanceSettings;
  paused?: boolean;
  out?: boolean;
}

export function Wallpaper(props: Props) {
  const { definition, config } = props;

  let element: ComponentChildren = null;
  if (SUPPORTED_IMAGE_WALLPAPER_EXTENSIONS.some((ext) => definition.filename?.endsWith(ext))) {
    element = <ImageWallpaper {...props} />;
  }

  if (SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS.some((ext) => definition.filename?.endsWith(ext))) {
    element = <VideoWallpaper {...props} />;
  }

  return (
    <div className={cs.container}>
      {element || <ThemedWallpaper {...props} />}
      {config.withOverlay && (
        <div
          className={cs.overlay}
          style={{ mixBlendMode: config.overlayMixBlendMode, backgroundColor: config.overlayColor }}
        />
      )}
    </div>
  );
}

export function ThemedWallpaper({ out, config }: Pick<Props, 'out' | 'config'>) {
  return (
    <div
      className={cx(cs.wallpaper, 'themed-wallpaper', {
        'wallpaper-out': out,
      })}
      style={getWallpaperStyles(config)}
    >
      <BackgroundByLayersV2 />
    </div>
  );
}

function ImageWallpaper({ definition, config, out }: Props) {
  return (
    <img
      className={cx(cs.wallpaper, 'wallpaper', { 'wallpaper-out': out })}
      style={getWallpaperStyles(config)}
      src={convertFileSrc(definition.metadata.path + '\\' + definition.filename!)}
    />
  );
}

function VideoWallpaper({ definition, config, out, paused }: Props) {
  const ref = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (ref.current) {
      console.debug('wallpaper state changed:', paused);
      if (paused) {
        ref.current.pause();
      } else {
        ref.current.play();
      }
    }
  }, [paused]);

  function onWaiting() {
    if (ref.current) {
      console.debug('video waiting for data, seeking to 0');
      ref.current.currentTime = 0;
    }
  }

  return (
    <video
      className={cx(cs.wallpaper, 'wallpaper', { 'wallpaper-out': out })}
      style={getWallpaperStyles(config)}
      ref={ref}
      src={convertFileSrc(definition.metadata.path + '\\' + definition.filename!)}
      controls={false}
      muted
      autoPlay
      loop
      playsInline
      disableRemotePlayback
      onWaiting={onWaiting}
      playbackRate={getPlaybackRate(config.playbackSpeed)}
    />
  );
}
