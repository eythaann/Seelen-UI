<script lang="ts">
  import { Icon } from "./Icon/index.ts";
  import { t } from "../utils/i18n.ts";
  import { MediaCommand } from "libs/ui/utils/MediaCommand.ts";

  interface Props {
    target: string;
    playing: boolean | null;
    controls: { previous: boolean; togglePlayPause: boolean; next: boolean };
    onaction: (action: "previous" | "togglePlayPause" | "next") => Promise<unknown>;
  }

  let { target, playing, controls, onaction }: Props = $props();
  let pending = $state(false);
  let failed = $state(false);
  const command = new MediaCommand((state) => {
    pending = state.pending;
    failed = state.failed;
  });

  $effect(() => {
    command.setTarget(target);
  });

  async function send(action: "previous" | "togglePlayPause" | "next") {
    if (pending || !controls[action]) return;
    await command.run(() => onaction(action));
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
