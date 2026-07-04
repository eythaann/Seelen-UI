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
  let currentVolume = $state(output.volume * 100);
  let isDragging = $state(false);

  $effect(() => {
    if (!isDragging) currentVolume = output.volume * 100;
  });

  const setVolumeThrottled = throttle((deviceId: string, level: number) => {
    invoke(SeelenCommand.SetVolumeLevel, { deviceId, sessionId: null, level: level / 100 });
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
    max={100}
    step={1}
  />
  <span class="flyout-value-label">{Math.round(currentVolume)}%</span>
</div>
