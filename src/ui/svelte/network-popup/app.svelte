<script lang="ts">
  import { globalState } from "./state.svelte";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { RadioDeviceKind, type WlanBssEntry } from "@seelen-ui/lib/types";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "./i18n";
  import WlanEntry from "./components/WlanEntry.svelte";
  import HotspotEntry from "./components/HotspotEntry.svelte";
  import HotspotView from "./components/HotspotView.svelte";

  // Group and sort entries
  const { connected, known, unknown, hidden } = $derived.by(() => {
    const entries = globalState.wlanBssEntries;

    // Filter hidden networks (no SSID)
    let hidden = entries.filter((e) => !e.ssid).toSorted((a, b) => b.signal - a.signal);

    // Group entries by SSID
    let grouped = entries.reduce(
      (groups, entry) => {
        if (!entry.ssid) {
          return groups;
        }
        if (!groups[entry.ssid]) {
          groups[entry.ssid] = [entry];
          return groups;
        }
        groups[entry.ssid]!.push(entry);
        groups[entry.ssid]!.sort((e1, e2) => e2.signal - e1.signal);
        return groups;
      },
      {} as Record<string, [WlanBssEntry, ...WlanBssEntry[]]>,
    );

    // Sort groups by signal strength
    let sorted = Object.values(grouped).toSorted((a, b) => {
      let signalA = Math.max(...a.map((e) => e.signal));
      let signalB = Math.max(...b.map((e) => e.signal));
      return signalB - signalA;
    });

    let connected = sorted.find((group) => group.some((e) => e.connected));
    let known = sorted.filter(
      (group) => group.every((e) => !e.connected) && group.some((e) => e.known),
    );
    let unknown = sorted.filter(
      (group) => group.every((e) => !e.connected) && group.every((e) => !e.known),
    );

    return { connected, known, unknown, hidden };
  });

  const wifiRadio = $derived(globalState.wifiRadio);

  function openNetworkSettings() {
    invoke(SeelenCommand.OpenFile, { path: "ms-settings:network" });
  }

  async function toggleWifiRadio() {
    if (wifiRadio) {
      await invoke(SeelenCommand.SetRadioState, {
        kind: RadioDeviceKind.WiFi,
        enabled: !wifiRadio.isEnabled,
      });
    }
  }

  $effect(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class="slu-std-popover network-popup">
  {#if globalState.view === "hotspot"}
    <HotspotView onBack={() => (globalState.view = "main")} />
  {:else}
    {#if !wifiRadio}
      <div class="network-no-adapter">
        {$t("no_adapter")}
      </div>
    {:else}
      <div class="network-radio-control">
        <div class="network-radio-label">
          <Icon iconName="FaWifi" />
          <span>Wi-Fi</span>
        </div>
        <input
          type="checkbox"
          data-skin="switch"
          checked={wifiRadio.isEnabled}
          onchange={toggleWifiRadio}
        />
      </div>
      {#if globalState.hotspot}
        <div class="network-section">
          <div class="network-section-title">{$t("hotspot.title")}</div>
          <div class="network-section-entries">
            <HotspotEntry />
          </div>
        </div>
      {/if}
    {/if}

    {#if wifiRadio?.isEnabled}
      {#if connected}
        <div class="network-section">
          <div class="network-section-title">{$t("connected")}</div>
          <div class="network-section-entries">
            <WlanEntry group={connected} />
          </div>
        </div>
      {/if}

      {#if known.length > 0}
        <div class="network-section">
          <div class="network-section-title">{$t("saved")}</div>
          <div class="network-section-entries">
            {#each known as group (group[0].ssid)}
              <WlanEntry {group} />
            {/each}
          </div>
        </div>
      {/if}

      <div class="network-section">
        <div class="network-section-title">
          <span>{$t("available")}</span>
          {#if globalState.isScanning}
            <div class="network-scanning">
              <Icon iconName="TbRefresh" />
            </div>
          {/if}
        </div>
        <div class="network-section-entries">
          {#if unknown.length === 0 && hidden.length === 0}
            <div class="network-empty">{$t("not_found")}</div>
          {:else}
            {#each unknown as group (group[0].ssid)}
              <WlanEntry {group} />
            {/each}
            {#if hidden.length > 0}
              <WlanEntry group={hidden} />
            {/if}
          {/if}
        </div>
      </div>

      {#if wifiRadio?.isEnabled}
        <div class="network-footer">
          <button data-skin="transparent" onclick={openNetworkSettings}>
            {$t("more")}
          </button>
        </div>
      {/if}
    {/if}
  {/if}
</div>
