<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { RadioDeviceKind, type RadioDevice } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";
  import type { IconName } from "@icons";

  function getRadioIcon(kind: RadioDeviceKind): IconName {
    switch (kind) {
      case RadioDeviceKind.WiFi:
        return "IoWifiSharp";
      case RadioDeviceKind.Bluetooth:
        return "IoBluetooth";
      case RadioDeviceKind.MobileBroadband:
        return "IoPhonePortraitSharp";
      case RadioDeviceKind.FM:
        return "IoRadio";
      case RadioDeviceKind.Other:
        return "IoRadioButtonOnSharp";
    }
  }

  function getRadioLabel(kind: RadioDeviceKind): string {
    switch (kind) {
      case "WiFi":
        return "Wi-Fi";
      case "Bluetooth":
        return "Bluetooth";
      case "MobileBroadband":
        return "Mobile Broadband";
      case "FM":
        return "FM Radio";
      default:
        return "Unknown";
    }
  }

  async function toggleRadio(radio: RadioDevice) {
    await invoke(SeelenCommand.SetRadioState, {
      kind: radio.kind,
      enabled: !radio.is_enabled,
    });
  }
</script>

{#if state.radios.length > 0}
  <div class="radio-buttons-container">
    {#each state.radios as radio (radio.id)}
      <button
        class="radio-button"
        class:radio-button-enabled={radio.is_enabled}
        class:radio-button-disabled={!radio.is_enabled}
        onclick={() => toggleRadio(radio)}
        title={`${radio.name} - ${radio.is_enabled ? "Enabled" : "Disabled"}`}
      >
        <Icon iconName={getRadioIcon(radio.kind)} size={24} />
        <span class="radio-button-label">{getRadioLabel(radio.kind)}</span>
      </button>
    {/each}
  </div>
{/if}

