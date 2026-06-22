<script lang="ts">
  import BackgroundByLayers from "../../BackgroundByLayers/BackgroundByLayers.svelte";
  import type { BaseProps } from "../types";
  import { getWallpaperStyles } from "../utils";
  import wallState from "../state.svelte";
  import Default from "./Default.svelte";
  import DOMPurify from "dompurify";

  let { definition, config, onLoad }: Pick<BaseProps, "definition" | "config" | "onLoad"> =
    $props();

  let styleEl = $state<HTMLStyleElement>();

  let safeHtml = $derived(DOMPurify.sanitize(definition?.html || ""));
  let safeCss = $derived(definition?.css?.replace(/<\/style/gi, "<\\/style") || "");

  $effect(() => {
    if (!styleEl) return;
    styleEl.textContent = `@scope { ${safeCss} }`;
  });

  let onLoadCalled = false;
  $effect(() => {
    if (!onLoad || onLoadCalled) return;
    if (!definition || definition.type !== "MediaPlayer" || !wallState.player?.thumbnail) {
      onLoadCalled = true;
      onLoad();
    } else if (!wallState.fetchingThumbnail) {
      onLoadCalled = true;
      onLoad();
    }
  });
</script>

{#if !definition || (definition.type === "MediaPlayer" && !wallState.player)}
  <div id="@default/wallpaper" class={["wallpaper", "default-wallpaper"]}>
    <Default />
  </div>
{:else}
  <div id={definition.id} class="wallpaper" style={config ? getWallpaperStyles(config) : undefined}>
    <style bind:this={styleEl}></style>
    <BackgroundByLayers />
    {@html safeHtml}
  </div>
{/if}

<style>
  .wallpaper {
    position: relative;
    width: 100%;
    height: 100%;
  }
</style>
