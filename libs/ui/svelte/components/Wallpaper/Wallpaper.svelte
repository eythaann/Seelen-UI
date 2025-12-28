<script lang="ts">
  import { WallpaperKind } from "@seelen-ui/lib/types";

  import ThemedWallpaper from "./components/ThemedWallpaper.svelte";
  import ImageWallpaper from "./components/ImageWallpaper.svelte";
  import VideoWallpaper from "./components/VideoWallpaper.svelte";
  import type { BaseProps } from "./types";
  import { defaultWallpaperConfig } from "./utils";

  let {
    definition,
    config = defaultWallpaperConfig,
    onLoad,
    out,
    pausedMessage,
    paused,
    static: staticProp,
    muted,
  }: BaseProps = $props();

  let loaded = $state(false);

  function handleLoad() {
    loaded = true;
    onLoad?.();
  }
</script>

<div class="wallpaper-container" class:rendering={loaded} class:will-unrender={out}>
  {#if definition?.type === WallpaperKind.Image}
    <ImageWallpaper {definition} {config} onLoad={handleLoad} />
  {:else if definition?.type === WallpaperKind.Video}
    {#if staticProp && definition.thumbnailFilename}
      <ImageWallpaper
        definition={{ ...definition, filename: definition.thumbnailFilename }}
        {config}
        onLoad={handleLoad}
      />
    {:else}
      <VideoWallpaper {definition} {config} {muted} {paused} onLoad={handleLoad} />
    {/if}
  {:else if definition?.type === WallpaperKind.Layered}
    <ThemedWallpaper {definition} {config} onLoad={handleLoad} />
  {:else}
    <ThemedWallpaper onLoad={handleLoad} />
  {/if}

  {#if config.withOverlay && loaded}
    <div
      class="wallpaper-overlay"
      style={`mix-blend-mode: ${config.overlayMixBlendMode}; background-color: ${config.overlayColor};`}
    ></div>
  {/if}

  {#if pausedMessage && paused && loaded && definition?.type === "Video"}
    <div class="paused-message">{pausedMessage}</div>
  {/if}
</div>

<style>
  .wallpaper-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .wallpaper-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .paused-message {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);

    background-color: rgba(0, 0, 0, 0.5);
    color: white;
    padding: 12px 20px;
    font-size: 14px;
    font-weight: 500;
    border-radius: 10px;
    backdrop-filter: blur(10px);
    z-index: 1000;
  }
</style>
