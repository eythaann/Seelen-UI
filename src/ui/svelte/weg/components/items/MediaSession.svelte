<script lang="ts">
  import { DEFAULT_THUMBNAIL } from "../../constants.ts";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { SeelenWegSide } from "@seelen-ui/lib/types";
  import { FileIcon, Icon } from "libs/ui/svelte/components/Icon/index.ts";
  import { convertFileSrc, invoke as tauriInvoke } from "@tauri-apps/api/core";
  import { t } from "../../i18n/index.ts";
  import type { MediaWegItem } from "../../types.ts";
  import { settingsState } from "../../state/settings.svelte.ts";
  import { getMenuForItem } from "../../generalMenu.ts";
  import { players } from "../../state/getters.svelte.ts";
  import { calcLuminance } from "../../application.ts";

  interface Props {
    item: MediaWegItem;
  }

  let { item }: Props = $props();

  const MAX_LUMINANCE = 210;
  const MIN_LUMINANCE = 40;
  const BRIGHTNESS_MULTIPLIER = 1.5;

  let luminance = $state(150);

  const session = $derived(players.value.find((s) => s.default));
  const thumbnailSrc = $derived(
    convertFileSrc(session?.thumbnail ? session.thumbnail : DEFAULT_THUMBNAIL),
  );
  const isHorizontal = $derived(
    settingsState.position === SeelenWegSide.Bottom || settingsState.position === SeelenWegSide.Top,
  );

  const filteredLuminance = $derived(
    Math.max(Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE), MIN_LUMINANCE),
  );
  const textColor = $derived(filteredLuminance < 125 ? "#efefef" : "#222222");

  $effect(() => {
    calcLuminance(thumbnailSrc)
      .then((l) => (luminance = l))
      .catch(console.error);
  });

  function onContextMenu(e: MouseEvent) {
    e.stopPropagation();
    const alignX = settingsState.popupAlignX;
    const alignY = settingsState.popupAlignY;
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getMenuForItem($t, item), alignX, alignY },
      forwardTo: null,
    });
  }

  function onClickBtn(cmd: string) {
    if (session) {
      tauriInvoke(cmd, { id: session.umid }).catch(console.error);
    }
  }

  const tooltip = $derived(
    session ? `${session.title}\n${session.author}` : $t("media.label"),
  );
</script>

<div
  role="group"
  class="weg-item media-session-container"
  class:media-session-container-horizontal={isHorizontal}
  class:media-session-container-vertical={!isHorizontal}
  data-tooltip={tooltip}
  data-tooltip-align-x={settingsState.popupAlignX}
  data-tooltip-align-y={settingsState.popupAlignY}
  oncontextmenu={onContextMenu}
>
  <div
    class="media-session"
    style="background-color: rgb({filteredLuminance}, {filteredLuminance}, {filteredLuminance})"
  >
    <div class="media-session-blurred-thumbnail-container">
      <img class="media-session-blurred-thumbnail" src={thumbnailSrc} loading="lazy" alt="" />
    </div>

    <div class="media-session-thumbnail-container">
      <img class="media-session-thumbnail" src={thumbnailSrc} loading="lazy" alt="" />
      {#if session}
        <FileIcon class="media-session-app-icon" umid={session.umid} />
      {/if}
    </div>

    <div class="media-session-info">
      <span
        class="media-session-title"
        class:media-session-title-default={!session}
        style="color: {textColor}"
      >
        {session ? session.title : $t("media.not_playing")}
      </span>

      {#if session}
        <div class="media-session-actions">
          <button data-skin="transparent" onclick={() => onClickBtn("media_prev")}>
            <Icon iconName="IoPlaySkipBack" color={textColor} size={12} />
          </button>
          <button data-skin="transparent" onclick={() => onClickBtn("media_toggle_play_pause")}>
            <Icon iconName={session?.playing ? "IoPause" : "IoPlay"} color={textColor} size={12} />
          </button>
          <button data-skin="transparent" onclick={() => onClickBtn("media_next")}>
            <Icon iconName="IoPlaySkipForward" color={textColor} size={12} />
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .media-session {
    display: grid;
    position: relative;
    width: 100%;
    height: 100%;
  }

  .media-session-container-horizontal .media-session {
    grid-template-columns: var(--config-item-size) 1fr;
  }

  .media-session-container-vertical .media-session {
    grid-template-rows: var(--config-item-size) 1fr;
  }

  .media-session-blurred-thumbnail-container {
    position: absolute;
    overflow: hidden;
    width: 100%;
    height: 100%;
  }

  .media-session-blurred-thumbnail {
    width: 100%;
    height: 100%;
    object-fit: fill;
    filter: blur(10px) brightness(125%) contrast(125%);
  }

  .media-session-thumbnail-container {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .media-session-thumbnail {
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #0004;
  }

  :global(.media-session-app-icon) {
    position: absolute;
    width: 25%;
    aspect-ratio: 1/1;
    right: 5%;
    bottom: 5%;
    object-fit: contain;
  }

  .media-session-info {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    overflow: hidden;
    padding: 4px;
  }

  .media-session-title {
    letter-spacing: 0.3px;
    line-height: 1.3em;
    font-size: 0.7rem;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
    overflow: hidden;
    max-width: 100%;
    margin-bottom: 2px;
    margin-right: -2px;
  }

  .media-session-title-default {
    text-align: center;
    white-space: normal;
  }

  .media-session-container-vertical .media-session-title {
    display: none;
  }

  .media-session-actions {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
  }

  .media-session-container-vertical .media-session-actions {
    flex-direction: column;
    gap: 12px;
  }

  .media-session-actions button {
    height: 16px;
    color: inherit;
  }
</style>
