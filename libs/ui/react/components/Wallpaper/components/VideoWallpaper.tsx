import { cx } from "libs/ui/react/utils/styling.ts";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef } from "preact/hooks";

import type { DefinedWallProps } from "../types";
import { getPlaybackRate, getWallpaperStyles } from "../utils.ts";
import cs from "../index.module.css";

const MAX_RETRIES = 3;
const WAITING_TIMEOUT_MS = 3000;
const STALL_CHECK_INTERVAL_MS = 5000;

export function VideoWallpaper({ definition, config, muted, paused, onLoad }: DefinedWallProps) {
  const ref = useRef<HTMLVideoElement>(null);
  const waitingTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const retryCountRef = useRef(0);
  const lastTimeUpdateRef = useRef(0);
  const isLoadedRef = useRef(false);

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

  // Cleanup on unmount to prevent memory leaks
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

  // Handle pause/play state changes
  useEffect(() => {
    if (ref.current && paused !== undefined) {
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
