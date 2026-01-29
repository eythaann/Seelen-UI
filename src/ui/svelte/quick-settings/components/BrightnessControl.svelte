<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";
  import { brightnessIcon } from "libs/ui/utils";
  import { throttle } from "lodash";

  let brightnessValue = $derived(state.brightness?.current ?? 0);

  const setBrightnessThrottled = throttle((brightness: number) => {
    invoke(SeelenCommand.SetMainMonitorBrightness, { brightness });
  }, 100);
</script>

{#if state.brightness}
  <span class="quick-settings-label">Brightness</span>
  <div class="quick-settings-item">
    <button
      data-skin="transparent"
      onclick={() => {
        /* TODO: add auto brightness toggle */
      }}
    >
      <Icon iconName={brightnessIcon(brightnessValue)} />
    </button>
    <input
      type="range"
      data-skin="flat"
      value={brightnessValue}
      oninput={(e) => {
        setBrightnessThrottled(Number(e.currentTarget.value));
      }}
      min={state.brightness.min}
      max={state.brightness.max}
    />
  </div>
{/if}
