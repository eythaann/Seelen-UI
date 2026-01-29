<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";
  import { brightnessIcon } from "libs/ui/utils";
  import { throttle } from "lodash";

  const setBrightnessThrottled = throttle((instanceName: string, level: number) => {
    invoke(SeelenCommand.SetMonitorBrightness, { instanceName, level });
  }, 100);
</script>

{#each state.brightness as brightness}
  <span class="quick-settings-label">Brightness</span>
  <div class="quick-settings-item">
    <button data-skin="transparent">
      <Icon iconName={brightnessIcon(brightness.currentBrightness)} />
    </button>
    <input
      type="range"
      data-skin="flat"
      value={brightness.currentBrightness}
      oninput={(e) => {
        setBrightnessThrottled(brightness.instanceName, Number(e.currentTarget.value));
      }}
      min={brightness.availableLevels[0]}
      max={brightness.availableLevels[brightness.levels]}
    />
  </div>
{/each}
