<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";

  let brightnessValue = $derived(state.brightness?.current ?? 0);

  function brightnessIcon(brightness: number) {
    if (brightness >= 60) {
      return "TbBrightnessUp";
    }
    return brightness >= 30 ? "TbBrightnessDown" : "TbBrightnessDownFilled";
  }

  let brightnessTimeout: number | null = null;
  function onBrightnessChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const value = Number(target.value);

    if (state.brightness) {
      state.brightness = { ...state.brightness, current: value };

      // Throttle brightness changes
      if (brightnessTimeout) {
        clearTimeout(brightnessTimeout);
      }
      brightnessTimeout = setTimeout(() => {
        invoke(SeelenCommand.SetMainMonitorBrightness, { brightness: value });
      }, 100) as any;
    }
  }
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
      <Icon iconName={brightnessIcon(brightnessValue)} size={20} />
    </button>
    <input
      type="range"
      class="quick-settings-slider"
      value={brightnessValue}
      oninput={onBrightnessChange}
      min={state.brightness.min}
      max={state.brightness.max}
    />
  </div>
{/if}
