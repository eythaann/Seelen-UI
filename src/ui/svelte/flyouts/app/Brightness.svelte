<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { brightnessIcon } from "libs/ui/utils";
  import { throttle } from "lodash";
  import type { PhysicalMonitor } from "@seelen-ui/lib/types";

  interface Props {
    monitor: PhysicalMonitor;
    orientation: string;
  }

  let { monitor, orientation }: Props = $props();

  // svelte-ignore state_referenced_locally
  let currentBrightness = $state(monitor.brightness ?? 0);
  let isDragging = $state(false);

  $effect(() => {
    if (!isDragging) currentBrightness = monitor.brightness ?? 0;
  });

  const setBrightnessThrottled = throttle((id: string, brightness: number) => {
    invoke(SeelenCommand.SetMonitorBrightness, { id, brightness });
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
      setBrightnessThrottled(monitor.id, currentBrightness);
    }}
    min={0}
    max={100}
  />
  <span class="flyout-value-label">{Math.round(currentBrightness)}%</span>
</div>
