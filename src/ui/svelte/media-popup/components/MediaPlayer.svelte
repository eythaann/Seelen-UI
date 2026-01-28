<script lang="ts">
  import type { MediaPlayer } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon, FileIcon } from "libs/ui/svelte/components/Icon";
  import { path } from "@tauri-apps/api";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { defaultThumbnail } from "../state.svelte";

  interface Props {
    session: MediaPlayer;
  }

  let { session }: Props = $props();

  const MAX_LUMINANCE = 210;
  const MIN_LUMINANCE = 40;
  const BRIGHTNESS_MULTIPLIER = 1.5;

  let luminance = $state(0);

  let thumbnailSrc = $derived(convertFileSrc(session?.thumbnail || defaultThumbnail));

  function calcLuminance(imageUrl: string): Promise<number> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.crossOrigin = "Anonymous";
      img.src = imageUrl;

      img.onload = () => {
        const canvas = document.createElement("canvas");
        const ctx = canvas.getContext("2d");

        if (!ctx) {
          reject(new Error("Unable to get canvas context"));
          return;
        }

        canvas.width = img.width;
        canvas.height = img.height;
        ctx.drawImage(img, 0, 0, img.width, img.height);

        const imageData = ctx.getImageData(0, 0, img.width, img.height);
        const data = imageData.data;
        let totalLuminance = 0;

        for (let i = 0; i < data.length; i += 4) {
          const r = data[i] || 0;
          const g = data[i + 1] || 0;
          const b = data[i + 2] || 0;

          const lum = 0.299 * r + 0.587 * g + 0.114 * b;
          totalLuminance += lum;
        }

        const avgLuminance = totalLuminance / (data.length / 4);
        resolve(avgLuminance);
      };

      img.onerror = (err) => {
        reject(err);
      };
    });
  }

  $effect(() => {
    calcLuminance(thumbnailSrc)
      .then((lum) => (luminance = lum))
      .catch(console.error);
  });

  const filteredLuminance = $derived(
    Math.max(Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE), MIN_LUMINANCE),
  );

  const color = $derived(filteredLuminance < 125 ? "#efefef" : "#222222");

  function onClickBtn(cmd: SeelenCommand) {
    invoke(cmd, { id: session.umid }).catch(console.error);
  }
</script>

<div
  class="media-session"
  style:background-color="rgb({filteredLuminance}, {filteredLuminance}, {filteredLuminance})"
>
  <img class="media-session-blurred-thumbnail" src={thumbnailSrc} alt="" />
  <div class="media-session-thumbnail-container">
    <FileIcon class="media-session-app-icon" umid={session.umid} title={session.owner.name} />
    <img class="media-session-thumbnail" src={thumbnailSrc} alt="" />
  </div>

  <div class="media-session-info" style:color>
    <h4 class="media-session-title">{session.title}</h4>
    <span class="media-session-author">{session.author}</span>
    <div class="media-session-actions">
      <button data-skin="transparent" onclick={() => onClickBtn(SeelenCommand.MediaPrev)}>
        <Icon iconName="IoPlaySkipBack" {color} />
      </button>
      <button
        data-skin="transparent"
        onclick={() => onClickBtn(SeelenCommand.MediaTogglePlayPause)}
      >
        <Icon iconName={session.playing ? "IoPause" : "IoPlay"} {color} />
      </button>
      <button data-skin="transparent" onclick={() => onClickBtn(SeelenCommand.MediaNext)}>
        <Icon iconName="IoPlaySkipForward" {color} />
      </button>
    </div>
  </div>
</div>
