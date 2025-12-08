<script lang="ts">
  import { globalState } from "./state.svelte";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { RadioDeviceKind } from "@seelen-ui/lib/types";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "./i18n";
  import BluetoothDevice from "./components/BluetoothDevice.svelte";

  // Filter devices to avoid duplicates (same name, prefer non-LE version)
  const uniqueDevices = $derived.by(() => {
    const devices = globalState.devices;
    const filtered: typeof devices = [];
    const seen = new Set<string>();

    for (const device of devices) {
      const existing = devices.find(
        (d) => d.name === device.name && d.id !== device.id && !d.isLowEnergy
      );
      if (existing) {
        if (!device.isLowEnergy) {
          // This is the classic version, keep it
          if (!seen.has(device.name)) {
            filtered.push(device);
            seen.add(device.name);
          }
        }
        // Skip LE version if classic exists
      } else {
        // No duplicate, add it
        if (!seen.has(device.name)) {
          filtered.push(device);
          seen.add(device.name);
        }
      }
    }

    return filtered;
  });

  const connectedDevices = $derived(uniqueDevices.filter((d) => d.paired && d.connected));
  const pairedDevices = $derived(uniqueDevices.filter((d) => d.paired && !d.connected));
  const availableDevices = $derived(uniqueDevices.filter((d) => !d.paired));

  const bluetoothRadio = $derived(globalState.bluetoothRadio);

  function openBluetoothSettings() {
    invoke(SeelenCommand.OpenFile, { path: "ms-settings:bluetooth" });
  }

  async function toggleBluetoothRadio() {
    if (bluetoothRadio) {
      await invoke(SeelenCommand.SetRadioState, {
        kind: RadioDeviceKind.Bluetooth,
        enabled: !bluetoothRadio.is_enabled,
      });
    }
  }
</script>

<div class="slu-popover bluetooth-popup">
  {#if !bluetoothRadio}
    <div class="bluetooth-no-adapter">
      {$t("no_adapter")}
    </div>
  {:else}
    <div class="bluetooth-radio-control">
      <div class="bluetooth-radio-label">
        <Icon iconName="IoBluetooth" />
        <span>Bluetooth</span>
      </div>
      <label class="bluetooth-radio-switch">
        <input
          type="checkbox"
          data-skin="switch"
          checked={bluetoothRadio.is_enabled}
          onchange={toggleBluetoothRadio}
        />
      </label>
    </div>
  {/if}

  {#if bluetoothRadio?.is_enabled}
    {#if connectedDevices.length > 0}
      <div class="bt-list">
        <div class="bt-list-title">{$t("connected")}</div>
        <div class="bt-list-devices">
          {#each connectedDevices as device (device.id)}
            <BluetoothDevice {device} />
          {/each}
        </div>
      </div>
    {/if}

    {#if pairedDevices.length > 0}
      <div class="bt-list">
        <div class="bt-list-title">{$t("paired")}</div>
        <div class="bt-list-devices">
          {#each pairedDevices as device (device.id)}
            <BluetoothDevice {device} />
          {/each}
        </div>
      </div>
    {/if}

    <div class="bt-list">
      <div class="bt-list-title">
        <span>{$t("available")}</span>
        {#if globalState.isScanning}
          <div class="bluetooth-scanning">
            <Icon iconName="TbRefresh" />
          </div>
        {/if}
      </div>
      <div class="bt-list-devices">
        {#if availableDevices.length > 0}
          {#each availableDevices as device (device.id)}
            <BluetoothDevice {device} />
          {/each}
        {:else}
          <div class="bluetooth-empty">{$t("not_found")}</div>
        {/if}
      </div>
    </div>

    {#if bluetoothRadio?.is_enabled}
      <div class="bluetooth-footer">
        <button data-skin="transparent" onclick={openBluetoothSettings}>
          {$t("more")}
        </button>
      </div>
    {/if}
  {/if}
</div>

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }

  :global(#root) {
    width: min-content;
    height: min-content;
  }
</style>
