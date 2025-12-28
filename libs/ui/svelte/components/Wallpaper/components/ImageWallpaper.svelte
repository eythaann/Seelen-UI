<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { DefinedWallProps } from "../types";
  import { getWallpaperStyles } from "../utils";

  let { definition, config, onLoad }: DefinedWallProps = $props();

  const imageSrc = $derived(convertFileSrc(definition.metadata.path + "\\" + definition.filename!));

  function handleError(e: Event) {
    const target = e.target as HTMLImageElement;
    console.error("Image failed to load:", {
      src: imageSrc,
      naturalWidth: target.naturalWidth,
      naturalHeight: target.naturalHeight,
    });
  }
</script>

<img
  id={definition.id}
  class="wallpaper"
  style={getWallpaperStyles(config)}
  src={imageSrc}
  onload={onLoad}
  onerror={handleError}
  decoding="async"
  loading="eager"
  alt=""
/>

<style>
  :global(.wallpaper) {
    width: 100%;
    height: 100%;
  }
</style>
