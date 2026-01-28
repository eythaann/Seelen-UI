<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";
  import { throttle } from "lodash";

  let defaultOutput = $derived(state.mediaOutputs.find((d) => d.isDefaultMultimedia));
  let defaultInput = $derived(state.mediaInputs.find((d) => d.isDefaultMultimedia));

  const setVolumeThrottled = throttle((deviceId: string, level: number) => {
    invoke(SeelenCommand.SetVolumeLevel, {
      deviceId,
      sessionId: null,
      level,
    });
  }, 100);

  function toggleMute(deviceId: string) {
    invoke(SeelenCommand.MediaToggleMute, { deviceId, sessionId: null });
  }
</script>

{#if defaultInput || defaultOutput}
  <span class="quick-settings-label">Default Multimedia Volume</span>
{/if}

{#if defaultOutput}
  <div class="quick-settings-item">
    <button data-skin="transparent" onclick={() => toggleMute(defaultOutput!.id)}>
      <Icon
        iconName={defaultOutput.muted ? "IoVolumeMuteOutline" : "IoVolumeHighOutline"}
        size={20}
      />
    </button>
    <input
      type="range"
      data-skin="flat"
      value={defaultOutput.volume}
      oninput={(e) => {
        setVolumeThrottled(defaultOutput.id, Number(e.currentTarget.value));
      }}
      min={0}
      max={1}
      step={0.01}
    />
    <span class="quick-settings-percentage">
      {Math.round(defaultOutput.volume * 100)}%
    </span>
  </div>
{/if}

{#if defaultInput}
  <div class="quick-settings-item">
    <button data-skin="transparent" onclick={() => toggleMute(defaultInput!.id)}>
      <Icon iconName={defaultInput.muted ? "BiMicrophoneOff" : "BiMicrophone"} size={20} />
    </button>
    <input
      type="range"
      data-skin="flat"
      value={defaultInput.volume}
      oninput={(e) => {
        setVolumeThrottled(defaultInput.id, Number(e.currentTarget.value));
      }}
      min={0}
      max={1}
      step={0.01}
    />
    <span class="quick-settings-percentage">
      {Math.round(defaultInput.volume * 100)}%
    </span>
  </div>
{/if}
