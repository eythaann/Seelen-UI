<script lang="ts">
  import { onMount } from "svelte";

  import { nativeWallpaperSource } from "../nativeWallpaperSource.svelte";
  import type { DefinedWallProps } from "../types";
  import { getWallpaperStyles } from "../utils";
  import ThemedWallpaper from "./ThemedWallpaper.svelte";

  let { definition, config, onLoad }: DefinedWallProps = $props();
  onMount(() => nativeWallpaperSource.acquire());
</script>

{#if nativeWallpaperSource.imageSrc}
  <img
    id={definition.id}
    class="wallpaper"
    style={getWallpaperStyles(config)}
    src={nativeWallpaperSource.imageSrc}
    onload={onLoad}
    decoding="async"
    alt=""
  />
{:else if nativeWallpaperSource.unavailable}
  <!-- Solid color background or unreadable file: show the default wallpaper instead of nothing. -->
  <ThemedWallpaper {onLoad} />
{/if}

<style>
  .wallpaper {
    position: relative;
    width: 100%;
    height: 100%;
  }
</style>
