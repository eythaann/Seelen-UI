import { useSignal } from '@preact/signals';
import {
  SUPPORTED_IMAGE_WALLPAPER_EXTENSIONS,
  SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS,
  WallpaperConfiguration,
} from '@seelen-ui/lib';
import { Wallpaper, WallpaperInstanceSettings } from '@seelen-ui/lib/types';
import { cx } from '@shared/styles';
import { convertFileSrc } from '@tauri-apps/api/core';
import { ComponentChildren } from 'preact';
import { useEffect, useMemo, useRef } from 'preact/hooks';

import { BackgroundByLayersV2 } from '@shared/components/BackgroundByLayers/infra';

import { getPlaybackRate, getWallpaperStyles } from './utils';
import cs from './index.module.css';

interface BaseProps {
  definition?: Wallpaper;
  config?: WallpaperInstanceSettings;
  onLoad?: () => void;
  paused?: boolean;
  out?: boolean;
}

const defaultWallpaperConfig = await WallpaperConfiguration.default();

export function Wallpaper(props: BaseProps) {
  const { definition, config = defaultWallpaperConfig } = props;

  const $loaded = useSignal(false);

  function onLoad() {
    $loaded.value = true;
    props.onLoad?.();
  }

  let element: ComponentChildren = null;
  if (
    definition &&
    SUPPORTED_IMAGE_WALLPAPER_EXTENSIONS.some((ext) => definition.filename?.endsWith(ext))
  ) {
    element = <ImageWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
  }

  if (
    definition &&
    SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS.some((ext) => definition.filename?.endsWith(ext))
  ) {
    element = <VideoWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
  }

  if (!element) {
    element = <ThemedWallpaper {...props} onLoad={onLoad} config={config} />;
  }

  return (
    <div
      className={cx(cs.container, 'wallpaper-container', {
        rendering: $loaded.value,
        'will-unrender': props.out,
      })}
    >
      {element}
      {config.withOverlay && $loaded.value && (
        <div
          className={cx(cs.overlay, 'wallpaper-overlay')}
          style={{ mixBlendMode: config.overlayMixBlendMode, backgroundColor: config.overlayColor }}
        />
      )}
    </div>
  );
}

export function ThemedWallpaper({ config, onLoad }: Pick<DefinedWallProps, 'config' | 'onLoad'>) {
  useEffect(() => {
    onLoad?.();
  }, []);

  return (
    <div className={cx(cs.wallpaper, 'themed-wallpaper')} style={getWallpaperStyles(config)}>
      <BackgroundByLayersV2 />
    </div>
  );
}

interface DefinedWallProps extends BaseProps {
  definition: Wallpaper;
  config: WallpaperInstanceSettings;
}

function ImageWallpaper({ definition, config, onLoad }: DefinedWallProps) {
  return (
    <img
      id={definition.id}
      className={cx(cs.wallpaper, 'wallpaper')}
      style={getWallpaperStyles(config)}
      src={convertFileSrc(definition.metadata.path + '\\' + definition.filename!)}
      onLoad={onLoad}
    />
  );
}

function VideoWallpaper({ definition, config, paused, onLoad }: DefinedWallProps) {
  const ref = useRef<HTMLVideoElement>(null);

  const videoSrc = useMemo(
    () => convertFileSrc(definition.metadata.path + '\\' + definition.filename!),
    [definition.metadata.path, definition.filename],
  );

  useEffect(() => {
    // https://github.com/facebook/react/issues/15583
    // this is a workaround for a bug in js that causes memory leak on video elements
    return () => {
      if (ref.current) {
        ref.current.pause();
        ref.current.removeAttribute('src');
        ref.current.load();
        if (window.gc) {
          setTimeout(() => window.gc?.(), 100);
        }
      }
    };
  }, []);

  useEffect(() => {
    if (ref.current && paused !== undefined) {
      // console.debug('📺 Wallpaper state changed:', paused, 'for:', definition.id);
      if (paused) {
        ref.current.pause();
      } else if (ref.current.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) {
        ref.current.play();
      }
    }
  }, [paused]);

  return (
    <video
      id={definition.id}
      className={cx(cs.wallpaper, 'wallpaper')}
      style={getWallpaperStyles(config)}
      ref={ref}
      src={videoSrc}
      controls={false}
      muted
      autoPlay={!paused}
      loop
      playsInline
      disableRemotePlayback
      playbackRate={getPlaybackRate(config.playbackSpeed)}
      onLoadedMetadata={onLoad} // mark video as loaded on portrait load
      onWaiting={() => {
        // console.debug('video waiting for data');
      }}
      onCanPlay={() => {
        if (ref.current && !paused) {
          ref.current.play();
        }
      }}
    />
  );
}
