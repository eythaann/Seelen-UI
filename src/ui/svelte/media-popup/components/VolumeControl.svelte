<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { throttle } from "lodash";

  interface Props {
    value: number;
    icon: string;
    deviceId: string;
    sessionName?: string;
    sessionId?: string;
    onRightAction?: () => void;
    withPercentage?: boolean;
    mutedIcon?: string;
    muted?: boolean;
  }

  let {
    value,
    icon,
    deviceId,
    sessionName,
    sessionId,
    onRightAction,
    withPercentage = false,
    mutedIcon,
    muted = false,
  }: Props = $props();

  let internalValue = $state(0);

  $effect(() => {
    internalValue = value * 100;
  });

  const onExternalChange = throttle((value: number) => {
    invoke(SeelenCommand.SetVolumeLevel, {
      deviceId,
      sessionId: sessionId || null,
      level: value / 100,
    }).catch(console.error);
  }, 100);

  async function toggleMute() {
    await invoke(SeelenCommand.MediaToggleMute, {
      deviceId,
      sessionId: sessionId || null,
    });
  }
</script>

<div class="media-control-volume">
  <button data-skin="transparent" onclick={toggleMute} title={sessionName}>
    <Icon iconName={(muted && mutedIcon ? mutedIcon : icon) as any} />
  </button>

  <input
    type="range"
    data-skin="flat"
    value={internalValue}
    min={0}
    max={100}
    step={1}
    oninput={(e) => {
      internalValue = Number(e.currentTarget.value);
      onExternalChange(internalValue);
    }}
  />

  {#if onRightAction}
    <button data-skin="transparent" onclick={onRightAction}>
      <Icon iconName="RiEqualizerLine" />
    </button>
  {/if}
</div>
