<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { HotspotState, RadioDeviceKind, type RadioDevice } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { state } from "../state.svelte";
  import type { IconName } from "libs/ui/icons";
  import { t } from "../i18n";

  const hdrMonitors = $derived(state.monitors.filter((m) => m.hdr !== null && m.hdr !== undefined));
  const hdrEnabled = $derived(hdrMonitors.length > 0 && hdrMonitors.every((m) => m.hdr));

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
        return $t("mobile_broadband");
      case "FM":
        return $t("fm_radio");
      default:
        return $t("unknown");
    }
  }

  async function toggleRadio(radio: RadioDevice) {
    await invoke(SeelenCommand.SetRadioState, {
      kind: radio.kind,
      enabled: !radio.isEnabled,
    });
  }

  async function toggleHotspot() {
    if (!state.hotspot) return;
    await invoke(SeelenCommand.SetNetworkHotspotState, {
      enabled: state.hotspot.state !== HotspotState.on,
    });
  }

  async function toggleHdr() {
    const newState = !hdrEnabled;
    await Promise.all(
      hdrMonitors.map((monitor) =>
        invoke(SeelenCommand.SetMonitorHdr, { id: monitor.id, state: newState }),
      ),
    );
  }

  async function toggleDarkMode() {
    await invoke(SeelenCommand.SystemSetDarkMode, { enabled: !state.darkMode });
  }

  async function toggleNightLight() {
    const enabled = !state.nightLightEnabled;
    invoke(SeelenCommand.SystemSetNightLightEnabled, { enabled }).then(() => {
      state.nightLightEnabled = enabled;
    });
  }
</script>

<div class="radio-buttons-container">
  {#each state.radios as radio (radio.id)}
    <button
      class="radio-button"
      data-skin={radio.isEnabled ? "solid" : "default"}
      onclick={() => toggleRadio(radio)}
      title={`${radio.name} - ${radio.isEnabled ? $t("enabled") : $t("disabled")}`}
    >
      <Icon iconName={getRadioIcon(radio.kind)} />
      <span class="radio-button-label">{getRadioLabel(radio.kind)}</span>
    </button>
  {/each}

  {#if state.hotspot}
    <button
      class="radio-button"
      data-skin={state.hotspot.state === HotspotState.on ? "solid" : "default"}
      disabled={state.hotspot.state === HotspotState.inTransition}
      onclick={toggleHotspot}
      title={`${$t("hotspot")} - ${state.hotspot.state === HotspotState.on ? $t("enabled") : $t("disabled")}`}
    >
      <Icon iconName="MdWifiTethering" />
      <span class="radio-button-label">{$t("hotspot")}</span>
    </button>
  {/if}

  {#if hdrMonitors.length > 0}
    <button
      class="radio-button"
      data-skin={hdrEnabled ? "solid" : "default"}
      onclick={toggleHdr}
      title={`${$t("hdr")} -${hdrEnabled ? $t("enabled") : $t("disabled")}`}
    >
      <Icon iconName="TbHdr" />
      <span class="radio-button-label">{$t("hdr")}</span>
    </button>
  {/if}

  <button
    class="radio-button"
    data-skin={state.darkMode ? "solid" : "default"}
    onclick={toggleDarkMode}
  >
    <Icon iconName={state.darkMode ? "IoMoon" : "IoSunny"} />
    <span class="radio-button-label">{state.darkMode ? $t("dark_mode") : $t("light_mode")}</span>
  </button>

  <button
    class="radio-button"
    data-skin={state.nightLightEnabled ? "solid" : "default"}
    onclick={toggleNightLight}
    title={`${$t("night_light")} - ${state.nightLightEnabled ? $t("enabled") : $t("disabled")}`}
  >
    <Icon iconName="IoEye" />
    <span class="radio-button-label">{$t("eye_care")}</span>
  </button>
</div>
