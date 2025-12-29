<script lang="ts">
  import type { BluetoothDevice, DevicePairingNeededAction } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "../i18n";
  import { getIconForBTDevice, getMinorAsString } from "../icons";
  import { globalState } from "../state.svelte";
  import { useTransition } from "libs/ui/svelte/utils/hooks.svelte";

  interface Props {
    device: BluetoothDevice;
  }
  let { device }: Props = $props();

  const transition = useTransition();

  let selected = $derived(globalState.selectedDeviceId === device.id);
  let loading = $derived(transition.loading);
  let error = $derived(transition.error);

  let pairingAction = $state<DevicePairingNeededAction | null>(null);

  let pinInput = $state("");
  let usernameInput = $state("");
  let passwordInput = $state("");

  // Watch for selection changes to reset state
  $effect(() => {
    if (!selected) {
      handleCancelPair();
      transition.clearError();
    }
  });

  async function handleDisconnect() {
    transition.start(async () => {
      await invoke(SeelenCommand.DisconnectBluetoothDevice, { id: device.id });
    });
  }

  async function handleForget() {
    transition.start(async () => {
      await invoke(SeelenCommand.ForgetBluetoothDevice, { id: device.id });
    });
  }

  async function handleTriggerPair() {
    if (pairingAction) {
      transition.start(async () => {
        await invoke(SeelenCommand.ConfirmBluetoothDevicePairing, {
          id: device.id,
          answer: {
            accept: true,
            pin: pinInput || null,
            username: usernameInput || null,
            password: passwordInput || null,
          },
        });
        pairingAction = null;
      });
    } else {
      transition.start(async () => {
        pairingAction = await invoke(SeelenCommand.RequestPairBluetoothDevice, {
          id: device.id,
        });
      });
    }

    pinInput = "";
    usernameInput = "";
    passwordInput = "";
  }

  async function handleCancelPair() {
    if (pairingAction) {
      await invoke(SeelenCommand.ConfirmBluetoothDevicePairing, {
        id: device.id,
        answer: {
          accept: false,
          pin: null,
          username: null,
          password: null,
        },
      }).catch((e) => {
        console.error("Cancel pairing error:", e);
      });
    }

    pairingAction = null;
    pinInput = "";
    usernameInput = "";
    passwordInput = "";
  }

  function getPairingMessage(): string {
    if (!pairingAction) return "";
    switch (pairingAction.needs) {
      case "ProvidePin":
        return $t("provide_pin");
      case "ConfirmPinMatch":
        return $t("confirm_pin");
      case "DisplayPin":
        return $t("display_pin");
      case "ConfirmOnly":
        return $t("confirm_only");
      case "ProvidePasswordCredential":
        return $t("provide_password");
      case "ProvideAddress":
        return $t("provide_address");
    }
  }

  function showPairingFields(): boolean {
    if (!selected || !pairingAction || loading) {
      return false;
    }
    const action = pairingAction;
    return (
      action.needs === "ProvidePin" ||
      action.needs === "ConfirmPinMatch" ||
      action.needs === "DisplayPin" ||
      action.needs === "ProvidePasswordCredential"
    );
  }

  function getDeviceTooltip(): string {
    if (device.appearance) {
      return `${device.appearance.category} - ${device.appearance.subcategory}`;
    }
    return `${device.majorClass} - ${getMinorAsString(device.minorClass)}`;
  }
</script>

<div
  class="bt-device"
  class:bt-device-selected={selected}
  onclick={() => {
    if (!selected) {
      globalState.selectedDeviceId = device.id;
    }
  }}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      globalState.selectedDeviceId = device.id;
    }
  }}
>
  <div class="bt-device-info">
    <Icon iconName={device.connected ? "TbBluetoothConnected" : "TbBluetooth"} />
    <span class="bt-device-name">{device.name}</span>
    <Icon iconName={getIconForBTDevice(device) as any} title={getDeviceTooltip()} />
    {#if device.isLowEnergy}
      <Icon iconName="MdOutlineEnergySavingsLeaf" title={$t("lowenergy")} />
    {/if}
  </div>

  {#if selected}
    {#if loading && !device.paired}
      <div class="bt-device-loading">{$t("pairing")}</div>
    {/if}

    {#if showPairingFields()}
      <div class="bt-device-pairing">
        <div class="bt-device-pairing-message">
          {getPairingMessage()}
        </div>

        {#if pairingAction?.needs === "ProvidePin"}
          <input
            type="text"
            data-skin="default"
            class:error
            bind:value={pinInput}
            placeholder="PIN"
          />
        {/if}

        {#if pairingAction?.needs === "ConfirmPinMatch" || pairingAction?.needs === "DisplayPin"}
          <div class="bt-device-pin-display">
            {pairingAction.pin || ""}
          </div>
        {/if}

        {#if pairingAction?.needs === "ProvidePasswordCredential"}
          <input
            type="text"
            data-skin="default"
            class:error
            bind:value={usernameInput}
            placeholder={$t("username")}
          />
          <input
            type="password"
            data-skin="default"
            class:error
            bind:value={passwordInput}
            placeholder={$t("password")}
          />
        {/if}
      </div>
    {/if}

    {#if !loading}
      <div class="bt-device-actions">
        {#if device.paired}
          {#if device.canDisconnect}
            <button data-skin="default" onclick={handleDisconnect} disabled={loading}>
              {$t("disconnect")}
            </button>
          {/if}
          <button data-skin="default" onclick={handleForget} disabled={loading}>
            {$t("unpair")}
          </button>
        {:else}
          {#if pairingAction}
            <button data-skin="default" onclick={handleCancelPair} disabled={loading}>
              {$t("cancel")}
            </button>
          {/if}

          <button data-skin="solid" onclick={handleTriggerPair} disabled={loading}>
            {pairingAction ? $t("confirm") : $t("pair")}
          </button>
        {/if}
      </div>
    {/if}

    {#if error}
      <div class="bt-device-error">
        {$t("pairing_failed")}
      </div>
    {/if}
  {/if}
</div>
