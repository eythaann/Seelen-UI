<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import type { DefinedWallProps } from "../types";
  import { getPlaybackRate, getWallpaperStyles } from "../utils";

  let { definition, config, muted, paused, onLoad }: DefinedWallProps = $props();

  const MAX_RETRIES = 3;
  const WAITING_TIMEOUT_MS = 3000;
  const STALL_CHECK_INTERVAL_MS = 5000;
  const RETRY_RESET_MS = 15000;

  let videoElement: HTMLVideoElement | undefined = $state();
  let waitingTimeout: ReturnType<typeof setTimeout> | undefined;
  let retryResetTimeout: ReturnType<typeof setTimeout> | undefined;
  let stallCheckInterval: ReturnType<typeof setInterval> | undefined;

  let retryCount = 0;
  let lastTimeUpdate = 0;
  let isLoaded = false;
  // Guards against overlapping doReload() calls from waiting/stalled/stallCheck firing in parallel.
  let isReloading = false;
  // Carry the playback position across a load() call; restored in handleCanPlay.
  let pendingSeekTime: number | null = null;

  const videoSrc = $derived(convertFileSrc(definition.metadata.path + "\\" + definition.filename!));

  // Single reload path. play() is NOT called here — handleCanPlay does it once
  // the browser has buffered enough, avoiding DOMException from play()-on-resetting-element.
  function doReload() {
    if (!videoElement || isReloading) return;
    isReloading = true;
    pendingSeekTime = videoElement.currentTime > 0 ? videoElement.currentTime : null;
    videoElement.load();
  }

  // Single recovery path shared by all stall detectors.
  function tryRecover(source: string) {
    if (retryCount < MAX_RETRIES) {
      console.debug(`Video recovery [${source}], attempt ${retryCount + 1}/${MAX_RETRIES}`);
      retryCount++;
      doReload();
    } else {
      scheduleRetryReset();
    }
  }

  // After exhausting retries, reset after a delay so the video can recover
  // rather than staying black forever.
  function scheduleRetryReset() {
    if (retryResetTimeout) return;
    retryResetTimeout = setTimeout(() => {
      retryResetTimeout = undefined;
      retryCount = 0;
      if (!paused) doReload();
    }, RETRY_RESET_MS);
  }

  function startStallCheck() {
    if (stallCheckInterval) clearInterval(stallCheckInterval);
    stallCheckInterval = setInterval(() => {
      if (!videoElement || paused || !isLoaded || isReloading) return;
      const currentTime = videoElement.currentTime;
      if (
        Math.abs(currentTime - lastTimeUpdate) < 0.01 &&
        videoElement.readyState < HTMLMediaElement.HAVE_FUTURE_DATA
      ) {
        tryRecover("stall-check");
      }
      lastTimeUpdate = currentTime;
    }, STALL_CHECK_INTERVAL_MS);
  }

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
    if (waitingTimeout) clearTimeout(waitingTimeout);
    waitingTimeout = setTimeout(() => {
      waitingTimeout = undefined;
      tryRecover("waiting");
    }, WAITING_TIMEOUT_MS);
  }

  function handlePlaying() {
    if (waitingTimeout) {
      clearTimeout(waitingTimeout);
      waitingTimeout = undefined;
    }
    if (retryResetTimeout) {
      clearTimeout(retryResetTimeout);
      retryResetTimeout = undefined;
    }
    retryCount = 0;
    isReloading = false;
  }

  function handleStalled() {
    if (!videoElement) return;
    // stalled fires when the buffer is full too; only intervene if readyState is also low.
    if (videoElement.readyState >= HTMLMediaElement.HAVE_FUTURE_DATA) return;
    tryRecover("stalled");
  }

  function handleCanPlay() {
    isReloading = false;
    if (!videoElement) return;
    if (pendingSeekTime !== null) {
      videoElement.currentTime = pendingSeekTime;
      pendingSeekTime = null;
    }
    if (!paused) {
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
    isReloading = false;
    const target = e.target as HTMLVideoElement;
    const error = target.error;
    if (error) {
      console.error("Video error:", { code: error.code, message: error.message, src: videoSrc });
      if (error.code === MediaError.MEDIA_ERR_NETWORK) {
        setTimeout(() => tryRecover("network-error"), 1000);
      } else {
        scheduleRetryReset();
      }
    }
  }

  function handleTimeUpdate() {
    if (videoElement) lastTimeUpdate = videoElement.currentTime;
  }

  // Reset all recovery state when the video source changes.
  $effect(() => {
    videoSrc; // tracked dependency
    if (waitingTimeout) {
      clearTimeout(waitingTimeout);
      waitingTimeout = undefined;
    }
    if (retryResetTimeout) {
      clearTimeout(retryResetTimeout);
      retryResetTimeout = undefined;
    }
    retryCount = 0;
    lastTimeUpdate = 0;
    isLoaded = false;
    isReloading = false;
    pendingSeekTime = null;
    startStallCheck();
  });

  onDestroy(() => {
    // Cleanup on unmount to prevent memory leaks
    // https://github.com/facebook/react/issues/15583
    // this is a workaround for a bug in js that causes memory leak on video elements
    if (waitingTimeout) clearTimeout(waitingTimeout);
    if (retryResetTimeout) clearTimeout(retryResetTimeout);
    if (stallCheckInterval) clearInterval(stallCheckInterval);
    if (videoElement) {
      videoElement.pause();
      videoElement.removeAttribute("src");
      videoElement.load();
      if (globalThis.gc) setTimeout(() => globalThis.gc?.(), 100);
    }
  });

  $effect(() => {
    if (videoElement) {
      videoElement.playbackRate = getPlaybackRate(config.playbackSpeed);
    }
  });
</script>

<video
  bind:this={videoElement}
  id={definition.id}
  class="wallpaper"
  style={getWallpaperStyles(config)}
  src={videoSrc}
  crossOrigin="anonymous"
  controls={false}
  muted={muted || config.muted}
  autoplay={!paused}
  loop
  playsinline
  disableRemotePlayback
  disablepictureinpicture
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

  video {
    transform: translateZ(0);
    will-change: transform;
    backface-visibility: hidden;
  }
</style>
