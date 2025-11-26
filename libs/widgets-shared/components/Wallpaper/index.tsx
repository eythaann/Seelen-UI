import { useSignal } from "@preact/signals";
import {
  SUPPORTED_IMAGE_WALLPAPER_EXTENSIONS,
  SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS,
  WallpaperConfiguration,
} from "@seelen-ui/lib";
import type { Wallpaper, WallpaperInstanceSettings } from "@seelen-ui/lib/types";
import { cx } from "@shared/styles";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ComponentChildren } from "preact";
import { useEffect, useMemo, useRef } from "preact/hooks";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { getPlaybackRate, getWallpaperStyles } from "./utils.ts";
import cs from "./index.module.css";

interface BaseProps {
  definition?: Wallpaper;
  config?: WallpaperInstanceSettings;
  onLoad?: () => void;
  muted?: boolean;
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
    SUPPORTED_IMAGE_WALLPAPER_EXTENSIONS.some((ext) => definition.filename?.toLowerCase()?.endsWith(ext))
  ) {
    element = <ImageWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
  }

  if (
    definition &&
    SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS.some((ext) => definition.filename?.toLowerCase()?.endsWith(ext))
  ) {
    element = <VideoWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
  }

  if (!element) {
    element = <ThemedWallpaper {...props} onLoad={onLoad} config={config} />;
  }

  return (
    <div
      className={cx(cs.container, "wallpaper-container", {
        rendering: $loaded.value,
        "will-unrender": props.out,
      })}
    >
      {element}
      {config.withOverlay && $loaded.value && (
        <div
          className={cx(cs.overlay, "wallpaper-overlay")}
          style={{
            mixBlendMode: config.overlayMixBlendMode,
            backgroundColor: config.overlayColor,
          }}
        />
      )}
    </div>
  );
}

export function ThemedWallpaper({ config, onLoad }: Pick<DefinedWallProps, "config" | "onLoad">) {
  useEffect(() => {
    onLoad?.();
  }, []);

  return (
    <div className={cx(cs.wallpaper, "themed-wallpaper")} style={getWallpaperStyles(config)}>
      <BackgroundByLayersV2 />
    </div>
  );
}

interface DefinedWallProps extends BaseProps {
  definition: Wallpaper;
  config: WallpaperInstanceSettings;
}

function ImageWallpaper({ definition, config, onLoad }: DefinedWallProps) {
  const retryCountRef = useRef(0);
  const MAX_RETRIES = 2;

  const imageSrc = useMemo(
    () => convertFileSrc(definition.metadata.path + "\\" + definition.filename!),
    [definition.metadata.path, definition.filename],
  );

  const handleError = (e: Event) => {
    const target = e.target as HTMLImageElement;
    console.error("Image failed to load:", {
      src: imageSrc,
      naturalWidth: target.naturalWidth,
      naturalHeight: target.naturalHeight,
    });

    // Attempt retry for network issues
    if (retryCountRef.current < MAX_RETRIES) {
      retryCountRef.current++;
      console.debug(`Retrying image load (${retryCountRef.current}/${MAX_RETRIES})`);

      // Force reload by adding timestamp
      setTimeout(() => {
        const timestamp = Date.now();
        target.src = `${imageSrc}?retry=${timestamp}`;
      }, 1000);
    }
  };

  const handleLoad = () => {
    retryCountRef.current = 0; // Reset on successful load
    onLoad?.();
  };

  return (
    <img
      id={definition.id}
      className={cx(cs.wallpaper, "wallpaper")}
      style={getWallpaperStyles(config)}
      src={imageSrc}
      onLoad={handleLoad}
      onError={handleError}
      decoding="async"
      loading="eager"
    />
  );
}

function VideoWallpaper({ definition, config, muted, paused, onLoad }: DefinedWallProps) {
  const ref = useRef<HTMLVideoElement>(null);
  const waitingTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const retryCountRef = useRef(0);
  const lastTimeUpdateRef = useRef(0);
  const isLoadedRef = useRef(false);

  const MAX_RETRIES = 3;
  const WAITING_TIMEOUT_MS = 3000;
  const STALL_CHECK_INTERVAL_MS = 5000;

  const videoSrc = useMemo(
    () => convertFileSrc(definition.metadata.path + "\\" + definition.filename!),
    [definition.metadata.path, definition.filename],
  );

  // Monitor for stalls by checking timeupdate
  useEffect(() => {
    const checkInterval = setInterval(() => {
      if (ref.current && !paused && isLoadedRef.current) {
        const currentTime = ref.current.currentTime;

        // If time hasn't changed and video should be playing, it's stalled
        if (
          currentTime === lastTimeUpdateRef.current &&
          ref.current.readyState < HTMLMediaElement.HAVE_FUTURE_DATA &&
          retryCountRef.current < MAX_RETRIES
        ) {
          console.debug("Video appears stalled, attempting recovery");
          retryCountRef.current++;
          ref.current.load();
          ref.current.currentTime = currentTime;
          ref.current.play().catch((err) => {
            console.error("Failed to resume video after stall:", err);
          });
        }

        lastTimeUpdateRef.current = currentTime;
      }
    }, STALL_CHECK_INTERVAL_MS);

    return () => clearInterval(checkInterval);
  }, [paused]);

  useEffect(() => {
    // https://github.com/facebook/react/issues/15583
    // this is a workaround for a bug in js that causes memory leak on video elements
    return () => {
      if (waitingTimeoutRef.current) {
        clearTimeout(waitingTimeoutRef.current);
      }
      if (ref.current) {
        ref.current.pause();
        ref.current.removeAttribute("src");
        ref.current.load();
        if (globalThis.gc) {
          setTimeout(() => globalThis.gc?.(), 100);
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
        ref.current.play().catch((err) => {
          console.error("Failed to play video:", err);
        });
      }
    }
  }, [paused]);

  const handleWaiting = () => {
    // Clear any existing timeout
    if (waitingTimeoutRef.current) {
      clearTimeout(waitingTimeoutRef.current);
    }

    // Set a timeout to detect if truly stuck
    waitingTimeoutRef.current = setTimeout(() => {
      if (ref.current && retryCountRef.current < MAX_RETRIES) {
        console.debug(
          `Video stuck in waiting state, retry ${retryCountRef.current + 1}/${MAX_RETRIES}`,
        );
        retryCountRef.current++;

        const currentTime = ref.current.currentTime;

        // Full reload to recover from stuck state
        ref.current.load();

        // Restore position if it wasn't at the beginning
        if (currentTime > 0) {
          ref.current.currentTime = currentTime;
        }

        if (!paused) {
          ref.current.play().catch((err) => {
            console.error("Failed to resume video after waiting timeout:", err);
          });
        }
      }
    }, WAITING_TIMEOUT_MS);
  };

  const handlePlaying = () => {
    // Clear timeout when playing successfully
    if (waitingTimeoutRef.current) {
      clearTimeout(waitingTimeoutRef.current);
      waitingTimeoutRef.current = undefined;
    }
    // Reset retry count on successful playback
    retryCountRef.current = 0;
  };

  const handleStalled = () => {
    // Stalled = browser thinks it can play but isn't fetching data
    if (ref.current && retryCountRef.current < MAX_RETRIES) {
      console.debug("Video network stalled, forcing reload");
      retryCountRef.current++;

      const currentTime = ref.current.currentTime;
      ref.current.load();
      ref.current.currentTime = currentTime;

      if (!paused) {
        ref.current.play().catch((err) => {
          console.error("Failed to resume video after stall:", err);
        });
      }
    }
  };

  const handleCanPlay = () => {
    if (ref.current && !paused) {
      ref.current.play().catch((err) => {
        console.error("Failed to play video on canplay:", err);
      });
    }
  };

  const handleLoadedMetadata = () => {
    isLoadedRef.current = true;
    onLoad?.();
  };

  const handleError = (e: Event) => {
    const target = e.target as HTMLVideoElement;
    const error = target.error;

    if (error) {
      console.error("Video error:", {
        code: error.code,
        message: error.message,
        src: videoSrc,
      });

      // Attempt recovery on certain errors
      if (error.code === MediaError.MEDIA_ERR_NETWORK && retryCountRef.current < MAX_RETRIES) {
        console.debug("Network error, attempting recovery");
        retryCountRef.current++;
        setTimeout(() => {
          if (ref.current) {
            ref.current.load();
            if (!paused) {
              ref.current.play().catch((err) => {
                console.error("Failed to recover from network error:", err);
              });
            }
          }
        }, 1000);
      }
    }
  };

  const handleTimeUpdate = () => {
    if (ref.current) {
      lastTimeUpdateRef.current = ref.current.currentTime;
    }
  };

  return (
    <video
      id={definition.id}
      className={cx(cs.wallpaper, "wallpaper")}
      style={getWallpaperStyles(config)}
      ref={ref}
      src={videoSrc}
      controls={false}
      muted={muted || config.muted}
      autoPlay={!paused}
      loop
      playsInline
      disableRemotePlayback
      preload="auto"
      playbackRate={getPlaybackRate(config.playbackSpeed)}
      onLoadedMetadata={handleLoadedMetadata}
      onWaiting={handleWaiting}
      onPlaying={handlePlaying}
      onStalled={handleStalled}
      onCanPlay={handleCanPlay}
      onError={handleError}
      onTimeUpdate={handleTimeUpdate}
    />
  );
}
