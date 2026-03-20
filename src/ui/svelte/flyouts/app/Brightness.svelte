<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { brightnessIcon } from "libs/ui/utils";
  import { throttle } from "lodash";
  import type { MonitorBrightness } from "@seelen-ui/lib/types";

  interface Props {
    brightness: MonitorBrightness;
    orientation: string;
  }

  let { brightness, orientation }: Props = $props();

  const setBrightnessThrottled = throttle((instanceName: string, level: number) => {
    invoke(SeelenCommand.SetMonitorBrightness, { instanceName, level });
  }, 100);
</script>

<div class="brightness">
  <Icon iconName={brightnessIcon(brightness.currentBrightness)} />
  <input
    type="range"
    data-skin="flat"
    data-orientation={orientation}
    value={brightness.currentBrightness}
    oninput={(e) => {
      setBrightnessThrottled(brightness.instanceName, Number(e.currentTarget.value));
    }}
    min={brightness.availableLevels[0]}
    max={brightness.availableLevels[brightness.levels]}
  />
</div>
