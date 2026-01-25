<script lang="ts">
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Icon, FileIcon } from "libs/ui/svelte/components/Icon";
  import VolumeControl from "./VolumeControl.svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const device = $derived(globalState.selectedDevice);

  const sessions = $derived.by(() => {
    if (!device || !device.sessions) return [];

    const sessionsIds = new Set();
    return device.sessions
      .toSorted((a, b) => a.name.localeCompare(b.name))
      .filter((s) => {
        const previousContains = sessionsIds.has(s.id);
        sessionsIds.add(s.id);
        return !previousContains;
      });
  });

  const iconName = $derived(device?.type === "input" ? "BiMicrophone" : "IoVolumeHighOutline");
  const mutedIconName = $derived(
    device?.type === "input" ? "BiMicrophoneOff" : "IoVolumeMuteOutline",
  );

  function openDeviceSettings() {
    if (device) {
      invoke(SeelenCommand.OpenFile, {
        path: `ms-settings:sound-properties?endpointId=${device.id}`,
      });
    }
  }
</script>

<div class="media-device-view">
  <div class="media-device-header">
    <button data-skin="transparent" onclick={onBack}>
      <Icon iconName="IoArrowBack" />
    </button>
    <span class="media-device-title">{device ? device.name : $t("device.missing")}</span>
  </div>

  {#if !!device}
    <span class="media-control-label">{$t("device.volume")}</span>
    <VolumeControl
      deviceId={device.id}
      value={device.volume}
      icon={device.muted ? mutedIconName : iconName}
      mutedIcon={mutedIconName}
      muted={device.muted}
    />

    {#if device.type !== "input" && sessions.length > 0}
      <span class="media-control-label">{$t("device.mixer")}</span>
      <div class="media-device-mixer">
        {#each sessions as channel (channel.id)}
          <div class="media-device-mixer-entry">
            <div class="media-device-mixer-entry-icon">
              {#if channel.isSystem}
                <Icon iconName="BsSpeaker" size={24} />
              {:else}
                <FileIcon path={channel.iconPath} />
              {/if}
            </div>
            <VolumeControl
              value={channel.volume}
              deviceId={device.id}
              sessionName={channel.isSystem ? $t("device.channel.system") : channel.name}
              sessionId={channel.id}
              icon={channel.muted ? mutedIconName : iconName}
              mutedIcon={mutedIconName}
              muted={channel.muted}
            />
          </div>
        {/each}
      </div>
    {/if}

    <div class="media-device-footer">
      <button data-skin="transparent" onclick={openDeviceSettings}>
        {$t("device.settings")}
      </button>
    </div>
  {/if}
</div>
