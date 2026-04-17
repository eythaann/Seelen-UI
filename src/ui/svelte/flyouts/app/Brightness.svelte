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

  // svelte-ignore state_referenced_locally
  let currentBrightness = $state(brightness.currentBrightness);
  let isDragging = $state(false);

  $effect(() => {
    if (!isDragging) currentBrightness = brightness.currentBrightness;
  });

  const setBrightnessThrottled = throttle((instanceName: string, level: number) => {
    invoke(SeelenCommand.SetMonitorBrightness, { instanceName, level });
  }, 100);
</script>

<div class="brightness">
  <Icon iconName={brightnessIcon(currentBrightness)} />
  <input
    type="range"
    data-skin="flat"
    data-orientation={orientation}
    value={currentBrightness}
    onpointerdown={() => (isDragging = true)}
    onpointerup={() => (isDragging = false)}
    oninput={(e) => {
      currentBrightness = Number(e.currentTarget.value);
      setBrightnessThrottled(brightness.instanceName, currentBrightness);
    }}
    min={brightness.availableLevels[0]}
    max={brightness.availableLevels[brightness.levels]}
  />
  <span class="flyout-value-label">{Math.round(currentBrightness)}%</span>
</div>
