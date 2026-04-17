<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { outputVolumeIcon } from "libs/ui/utils";
  import { throttle } from "lodash";
  import type { MediaDevice } from "@seelen-ui/lib/types";

  interface Props {
    output: MediaDevice;
    orientation: string;
  }

  let { output, orientation }: Props = $props();

  // svelte-ignore state_referenced_locally
  let currentVolume = $state(output.volume);
  let isDragging = $state(false);

  $effect(() => {
    if (!isDragging) currentVolume = output.volume;
  });

  const setVolumeThrottled = throttle((deviceId: string, level: number) => {
    invoke(SeelenCommand.SetVolumeLevel, { deviceId, sessionId: null, level });
  }, 100);

  function toggleMute(deviceId: string) {
    invoke(SeelenCommand.MediaToggleMute, { deviceId, sessionId: null });
  }
</script>

<div class="volume">
  <Icon
    iconName={outputVolumeIcon(output.muted, output.volume)}
    onclick={() => toggleMute(output.id)}
  />
  <input
    type="range"
    data-skin="flat"
    data-orientation={orientation}
    value={currentVolume}
    onpointerdown={() => (isDragging = true)}
    onpointerup={() => (isDragging = false)}
    oninput={(e) => {
      currentVolume = Number(e.currentTarget.value);
      setVolumeThrottled(output.id, currentVolume);
    }}
    min={0}
    max={1}
    step={0.01}
  />
  <span class="flyout-value-label">{Math.round(currentVolume * 100)}%</span>
</div>
