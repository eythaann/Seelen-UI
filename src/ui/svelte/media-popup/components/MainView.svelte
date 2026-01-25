<script lang="ts">
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import VolumeControl from "./VolumeControl.svelte";
  import MediaDevice from "./MediaDevice.svelte";
  import MediaPlayer from "./MediaPlayer.svelte";

  function setViewDeviceId(id: string) {
    globalState.selectedDeviceId = id;
    globalState.view = "mixer";
  }
</script>

<div class="media-main-view">
  {#if globalState.defaultOutput || globalState.defaultInput}
    <span class="media-control-label">
      {$t("default_multimedia_volume")}
    </span>

    {#if globalState.defaultOutput}
      <VolumeControl
        value={globalState.defaultOutput.volume}
        deviceId={globalState.defaultOutput.id}
        icon={globalState.defaultOutput.muted ? "IoVolumeMuteOutline" : "IoVolumeHighOutline"}
        mutedIcon="IoVolumeMuteOutline"
        muted={globalState.defaultOutput.muted}
        onRightAction={() => setViewDeviceId(globalState.defaultOutput!.id)}
      />
    {/if}

    {#if globalState.defaultInput}
      <VolumeControl
        value={globalState.defaultInput.volume}
        deviceId={globalState.defaultInput.id}
        icon={globalState.defaultInput.muted ? "BiMicrophoneOff" : "BiMicrophone"}
        mutedIcon="BiMicrophoneOff"
        muted={globalState.defaultInput.muted}
        onRightAction={() => setViewDeviceId(globalState.defaultInput!.id)}
      />
    {/if}
  {/if}

  {#if globalState.outputs.length > 0}
    <span class="media-control-label">
      {$t("output_devices")}
    </span>
    <div class="media-device-group">
      {#each globalState.outputs as device (device.id)}
        <MediaDevice {device} {setViewDeviceId} />
      {/each}
    </div>
  {/if}

  {#if globalState.inputs.length > 0}
    <span class="media-control-label">
      {$t("input_devices")}
    </span>
    <div class="media-device-group">
      {#each globalState.inputs as device (device.id)}
        <MediaDevice {device} {setViewDeviceId} />
      {/each}
    </div>
  {/if}

  {#if globalState.sessions.length > 0}
    <span class="media-control-label">{$t("players")}</span>
    <div class="media-control-session-list">
      {#each globalState.sessions as session (session.umid)}
        <MediaPlayer {session} />
      {/each}
    </div>
  {/if}
</div>
