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
    value={output.volume}
    oninput={(e) => {
      setVolumeThrottled(output.id, Number(e.currentTarget.value));
    }}
    min={0}
    max={1}
    step={0.01}
  />
</div>
