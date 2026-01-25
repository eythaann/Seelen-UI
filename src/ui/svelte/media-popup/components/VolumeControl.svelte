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
    internalValue = value;
  });

  const onExternalChange = throttle((value: number) => {
    invoke(SeelenCommand.SetVolumeLevel, {
      deviceId,
      sessionId: sessionId || null,
      level: value,
    }).catch(console.error);
  }, 100);

  function onInternalChange(value: number) {
    internalValue = value;
    onExternalChange(value);
  }

  function handleSliderChange(e: Event) {
    const target = e.target as HTMLInputElement;
    onInternalChange(Number(target.value));
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const isUp = e.deltaY < 0;
    const level = Math.max(0, Math.min(1, internalValue + (isUp ? 0.02 : -0.02)));
    onInternalChange(level);
  }

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
    max={1}
    step={0.01}
    onchange={handleSliderChange}
  />

  {#if onRightAction}
    <button data-skin="transparent" onclick={onRightAction}>
      <Icon iconName="RiEqualizerLine" />
    </button>
  {/if}
</div>
