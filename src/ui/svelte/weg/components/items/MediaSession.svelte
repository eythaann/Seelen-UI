<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon, Icon, SpecificIcon } from "libs/ui/svelte/components/Icon/index.ts";
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

  const session = $derived(players.value.find((s) => s.default));
  const thumbnailSrc = $derived(session?.thumbnail ? convertFileSrc(session.thumbnail) : null);

  let luminance = $state(150);
  const filteredLuminance = $derived(
    Math.max(Math.min(luminance * BRIGHTNESS_MULTIPLIER, MAX_LUMINANCE), MIN_LUMINANCE),
  );
  const textColor = $derived(filteredLuminance < 125 ? "#efefef" : "#222222");

  $effect(() => {
    if (!thumbnailSrc) {
      luminance = 150;
      return;
    }
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

  const tooltip = $derived(session ? `${session.title} - ${session.author}` : $t("media.label"));
</script>

<div
  role="group"
  class="weg-item weg-item-large media-session-container"
  data-tooltip={tooltip}
  data-tooltip-origin-y={settingsState.tooltipOrigin.y}
  data-tooltip-origin-x={settingsState.tooltipOrigin.x}
  data-tooltip-align-x={settingsState.popupAlignX}
  data-tooltip-align-y={settingsState.popupAlignY}
  oncontextmenu={onContextMenu}
>
  <div class="media-session">
    {#if thumbnailSrc}
      <div
        class="media-session-blurred-thumbnail-container"
        style:background-color={`rgb(${filteredLuminance}, ${filteredLuminance}, ${filteredLuminance})`}
      >
        <img class="media-session-blurred-thumbnail" src={thumbnailSrc} loading="lazy" alt="" />
      </div>
    {/if}

    <div class="media-session-thumbnail-container">
      {#if thumbnailSrc}
        <img class="media-session-thumbnail" src={thumbnailSrc} loading="lazy" alt="" />
      {:else}
        <SpecificIcon class="media-session-thumbnail" name="defaultPlayerThumbnail" />
      {/if}

      {#if session}
        <FileIcon class="media-session-app-icon" umid={session.umid} />
      {/if}
    </div>

    <div class="media-session-info">
      <span
        class="media-session-title"
        class:media-session-title-default={!session}
        style:color={session ? textColor : undefined}
      >
        {session ? session.title : $t("media.not_playing")}
      </span>

      {#if session}
        <div class="media-session-actions" style="color: {textColor}">
          <button data-skin="transparent" onclick={() => onClickBtn("media_prev")}>
            <Icon iconName="IoPlaySkipBack" size={12} />
          </button>
          <button data-skin="transparent" onclick={() => onClickBtn("media_toggle_play_pause")}>
            <Icon iconName={session?.playing ? "IoPause" : "IoPlay"} size={12} />
          </button>
          <button data-skin="transparent" onclick={() => onClickBtn("media_next")}>
            <Icon iconName="IoPlaySkipForward" size={12} />
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .media-session {
    display: flex;
    position: relative;
    width: 100%;
    height: 100%;

    :global(.vertical) & {
      flex-direction: column;
    }
  }

  .media-session-blurred-thumbnail-container {
    position: absolute;
    overflow: hidden;
    width: 100%;
    height: 100%;

    .media-session-blurred-thumbnail {
      width: 100%;
      height: 100%;
      object-fit: fill;
      filter: blur(10px) brightness(125%) contrast(125%);
    }
  }

  .media-session-thumbnail-container {
    position: relative;
    width: var(--config-item-size);
    height: var(--config-item-size);
    flex-shrink: 0;

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
  }

  .media-session-info {
    flex: 1;

    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;

    position: relative;
    overflow: hidden;
    padding: 4px;

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

      &.media-session-title-default {
        text-align: center;
        white-space: normal;
      }

      :global(.vertical) & {
        display: none;
      }
    }

    .media-session-actions {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 2px;

      :global(.vertical) & {
        flex-direction: column;
        gap: 12px;
      }

      button {
        font-size: 8px;
      }
    }
  }
</style>
