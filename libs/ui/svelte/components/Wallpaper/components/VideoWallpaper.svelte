<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import type { DefinedWallProps } from "../types";
  import { getPlaybackRate, getWallpaperStyles } from "../utils";

  let { definition, config, muted, paused, onLoad }: DefinedWallProps = $props();

  const MAX_RETRIES = 3;
  const WAITING_TIMEOUT_MS = 3000;
  const STALL_CHECK_INTERVAL_MS = 5000;

  let videoElement: HTMLVideoElement | undefined = $state();
  let waitingTimeout: ReturnType<typeof setTimeout> | undefined;
  let retryCount = 0;
  let lastTimeUpdate = 0;
  let isLoaded = false;
  let stallCheckInterval: ReturnType<typeof setInterval> | undefined;

  const videoSrc = $derived(convertFileSrc(definition.metadata.path + "\\" + definition.filename!));

  // Monitor for stalls by checking timeupdate
  function startStallCheck() {
    if (stallCheckInterval) {
      clearInterval(stallCheckInterval);
    }

    stallCheckInterval = setInterval(() => {
      if (videoElement && !paused && isLoaded) {
        const currentTime = videoElement.currentTime;

        // If time hasn't changed and video should be playing, it's stalled
        if (
          currentTime === lastTimeUpdate &&
          videoElement.readyState < HTMLMediaElement.HAVE_FUTURE_DATA &&
          retryCount < MAX_RETRIES
        ) {
          console.debug("Video appears stalled, attempting recovery");
          retryCount++;
          videoElement.load();
          videoElement.currentTime = currentTime;
          videoElement.play().catch((err) => {
            console.error("Failed to resume video after stall:", err);
          });
        }

        lastTimeUpdate = currentTime;
      }
    }, STALL_CHECK_INTERVAL_MS);
  }

  // Handle pause/play state changes
  $effect(() => {
    if (videoElement && paused !== undefined) {
      if (paused) {
        videoElement.pause();
      } else if (videoElement.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) {
        videoElement.play().catch((err) => {
          console.error("Failed to play video:", err);
        });
      }
    }
  });

  function handleWaiting() {
    // Clear any existing timeout
    if (waitingTimeout) {
      clearTimeout(waitingTimeout);
    }

    // Set a timeout to detect if truly stuck
    waitingTimeout = setTimeout(() => {
      if (videoElement && retryCount < MAX_RETRIES) {
        console.debug(`Video stuck in waiting state, retry ${retryCount + 1}/${MAX_RETRIES}`);
        retryCount++;

        const currentTime = videoElement.currentTime;

        // Full reload to recover from stuck state
        videoElement.load();

        // Restore position if it wasn't at the beginning
        if (currentTime > 0) {
          videoElement.currentTime = currentTime;
        }

        if (!paused) {
          videoElement.play().catch((err) => {
            console.error("Failed to resume video after waiting timeout:", err);
          });
        }
      }
    }, WAITING_TIMEOUT_MS);
  }

  function handlePlaying() {
    // Clear timeout when playing successfully
    if (waitingTimeout) {
      clearTimeout(waitingTimeout);
      waitingTimeout = undefined;
    }
    // Reset retry count on successful playback
    retryCount = 0;
  }

  function handleStalled() {
    // Stalled = browser thinks it can play but isn't fetching data
    if (videoElement && retryCount < MAX_RETRIES) {
      console.debug("Video network stalled, forcing reload");
      retryCount++;

      const currentTime = videoElement.currentTime;
      videoElement.load();
      videoElement.currentTime = currentTime;

      if (!paused) {
        videoElement.play().catch((err) => {
          console.error("Failed to resume video after stall:", err);
        });
      }
    }
  }

  function handleCanPlay() {
    if (videoElement && !paused) {
      videoElement.play().catch((err) => {
        console.error("Failed to play video on canplay:", err);
      });
    }
  }

  function handleLoadedMetadata() {
    isLoaded = true;
    onLoad?.();
  }

  function handleError(e: Event) {
    const target = e.target as HTMLVideoElement;
    const error = target.error;

    if (error) {
      console.error("Video error:", {
        code: error.code,
        message: error.message,
        src: videoSrc,
      });

      // Attempt recovery on certain errors
      if (error.code === MediaError.MEDIA_ERR_NETWORK && retryCount < MAX_RETRIES) {
        console.debug("Network error, attempting recovery");
        retryCount++;
        setTimeout(() => {
          if (videoElement) {
            videoElement.load();
            if (!paused) {
              videoElement.play().catch((err) => {
                console.error("Failed to recover from network error:", err);
              });
            }
          }
        }, 1000);
      }
    }
  }

  function handleTimeUpdate() {
    if (videoElement) {
      lastTimeUpdate = videoElement.currentTime;
    }
  }

  onMount(() => {
    startStallCheck();
  });

  onDestroy(() => {
    // Cleanup on unmount to prevent memory leaks
    // https://github.com/facebook/react/issues/15583
    // this is a workaround for a bug in js that causes memory leak on video elements
    if (waitingTimeout) {
      clearTimeout(waitingTimeout);
    }
    if (stallCheckInterval) {
      clearInterval(stallCheckInterval);
    }
    if (videoElement) {
      videoElement.pause();
      videoElement.removeAttribute("src");
      videoElement.load();
      if (globalThis.gc) {
        setTimeout(() => globalThis.gc?.(), 100);
      }
    }
  });

  $effect(() => {
    if (videoElement) {
      videoElement.playbackRate = getPlaybackRate(config.playbackSpeed)
    }
  })
</script>

<video
  bind:this={videoElement}
  id={definition.id}
  class="wallpaper"
  style={getWallpaperStyles(config)}
  src={videoSrc}
  controls={false}
  muted={muted || config.muted}
  autoplay={!paused}
  loop
  playsinline
  disableRemotePlayback
  preload="auto"
  onloadedmetadata={handleLoadedMetadata}
  onwaiting={handleWaiting}
  onplaying={handlePlaying}
  onstalled={handleStalled}
  oncanplay={handleCanPlay}
  onerror={handleError}
  ontimeupdate={handleTimeUpdate}
></video>

<style>
  :global(.wallpaper) {
    width: 100%;
    height: 100%;
  }
</style>
