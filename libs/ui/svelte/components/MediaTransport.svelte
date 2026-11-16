<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import type { MediaPlayerControls } from "@seelen-ui/lib/types";
  import { Icon } from "./Icon/index.ts";
  import { t } from "../utils/i18n.ts";
  import { MediaCommand } from "libs/ui/utils/MediaCommand.ts";

  type Action = "previous" | "togglePlayPause" | "next";

  interface Props {
    target: string;
    playing: boolean | null;
    controls: Pick<MediaPlayerControls, Action>;
    sessionId?: string;
    onaction?: (action: Action) => Promise<unknown>;
  }

  let { target, playing, controls, sessionId, onaction }: Props = $props();
  let pending = $state(false);
  let failed = $state(false);
  const command = new MediaCommand((state) => {
    pending = state.pending;
    failed = state.failed;
  });

  $effect(() => {
    command.setTarget(target);
  });

  async function send(action: Action) {
    if (pending || !controls[action]) return;
    await command.run(() => {
      if (onaction) return onaction(action);
      if (!sessionId) return Promise.reject(new Error("Media session is no longer available"));
      const commands = {
        previous: SeelenCommand.MediaPrev,
        togglePlayPause: SeelenCommand.MediaTogglePlayPause,
        next: SeelenCommand.MediaNext,
      } as const;
      return invoke(commands[action], { id: sessionId });
    });
  }
</script>

<div class="media-transport" aria-busy={pending}>
  <div class="media-session-actions">
    <button
      data-skin="transparent"
      disabled={pending || !controls.previous}
      aria-label={$t("media.previous")}
      title={$t(controls.previous ? "media.previous" : "media.unsupported")}
      onclick={() => send("previous")}
    ><Icon iconName="IoPlaySkipBack" /></button>
    <button
      data-skin="transparent"
      disabled={pending || !controls.togglePlayPause}
      aria-label={$t(playing === null ? "media.play_pause" : playing ? "media.pause" : "media.play")}
      title={$t(controls.togglePlayPause ? "media.play_pause" : "media.unsupported")}
      onclick={() => send("togglePlayPause")}
    ><Icon iconName={playing === null ? "HiMiniPlayPause" : playing ? "IoPause" : "IoPlay"} /></button>
    <button
      data-skin="transparent"
      disabled={pending || !controls.next}
      aria-label={$t("media.next")}
      title={$t(controls.next ? "media.next" : "media.unsupported")}
      onclick={() => send("next")}
    ><Icon iconName="IoPlaySkipForward" /></button>
  </div>
  {#if failed}
    <span class="media-transport-error" role="status">{$t("media.command_failed")}</span>
  {/if}
</div>
