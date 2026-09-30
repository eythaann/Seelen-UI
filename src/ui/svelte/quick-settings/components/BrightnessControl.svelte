<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";
  import { brightnessIcon } from "libs/ui/utils";
  import { throttle } from "lodash";

  const setBrightnessThrottled = throttle((id: string, brightness: number) => {
    invoke(SeelenCommand.SetMonitorBrightness, { id, brightness });
  }, 100);
</script>

{#each state.monitors as monitor (monitor.id)}
  {#if monitor.brightness != null}
    <span class="quick-settings-label">{monitor.name}</span>
    <div class="quick-settings-item">
      <button data-skin="transparent">
        <Icon iconName={brightnessIcon(monitor.brightness)} />
      </button>
      <input
        type="range"
        data-skin="flat"
        value={monitor.brightness}
        oninput={(e) => {
          setBrightnessThrottled(monitor.id, Number(e.currentTarget.value));
        }}
        min={0}
        max={100}
      />
    </div>
  {/if}
{/each}
