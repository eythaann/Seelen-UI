import { useSignalEffect } from '@preact/signals';
import { cx } from '@shared/styles';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useRef } from 'react';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { getPlaybackRate, getWallpaperStyles } from './application';

import { SUPPORTED_IMAGES, SUPPORTED_VIDEOS } from '../shared/constants';
import { $paused } from '../shared/state';

export interface Props {
  out?: boolean;
  path: string;
  onLoad?: () => void;
  onError?: () => void;
}

export function Wallpaper(props: Props) {
  if (SUPPORTED_IMAGES.some((ext) => props.path.endsWith(ext))) {
    return <ImageWallpaper {...props} />;
  }

  if (SUPPORTED_VIDEOS.some((ext) => props.path.endsWith(ext))) {
    return <VideoWallpaper {...props} />;
  }

  // fallback in case of unsupported file
  return <ThemedWallpaper {...props} />;
}

export function ThemedWallpaper({ out }: { out?: boolean }) {
  return (
    <div
      className={cx('themed-wallpaper', {
        'wallpaper-out': out,
      })}
      style={getWallpaperStyles()}
    >
      <BackgroundByLayersV2 />
    </div>
  );
}

function ImageWallpaper({ out, path, onLoad, onError }: Props) {
  return (
    <img
      className={cx('wallpaper', { 'wallpaper-out': out })}
      style={getWallpaperStyles()}
      src={convertFileSrc(path)}
      onLoad={onLoad}
      onError={onError}
    />
  );
}

function VideoWallpaper({ out, path, onLoad, onError }: Props) {
  const ref = useRef<HTMLVideoElement>(null);

  useSignalEffect(() => {
    if (ref.current) {
      console.debug('wallpaper state changed:', $paused.value);
      if ($paused.value) {
        ref.current.pause();
      } else {
        ref.current.play();
      }
    }
  });

  function onWaiting() {
    if (ref.current) {
      console.debug('video waiting for data, seeking to 0');
      ref.current.currentTime = 0;
    }
  }

  return (
    <video
      className={cx('wallpaper', { 'wallpaper-out': out })}
      style={getWallpaperStyles()}
      ref={ref}
      src={convertFileSrc(path)}
      muted
      autoPlay
      loop
      playsInline
      disableRemotePlayback
      onWaiting={onWaiting}
      onLoadedData={onLoad}
      onError={onError}
      playbackRate={getPlaybackRate()}
    />
  );
}
