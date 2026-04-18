<script lang="ts">
  import type { WlanBssEntry } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { untrack } from "svelte";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import type { IconName } from "libs/ui/icons";
  import { t } from "../i18n";
  import { useTransition } from "libs/ui/svelte/utils/hooks.svelte";
  import { globalState } from "../state.svelte";

  interface Props {
    group: WlanBssEntry[];
  }

  let { group }: Props = $props();

  const transition = useTransition();

  let entry = $derived(group[0]!);
  let isHiddenGroup = $derived(!entry.ssid);
  let selected = $derived(
    globalState.selectedSsid !== null &&
      (globalState.selectedSsid === entry.ssid ||
        (isHiddenGroup && globalState.selectedSsid === "__HIDDEN_SSID__")),
  );

  let loading = $derived(transition.loading);
  let error = $derived(transition.error);

  let forceShowFields = $state(false);
  let ssid = $state("");
  let password = $state("");
  let ssidInputRef: HTMLInputElement | undefined = $state();
  let passwordInputRef: HTMLInputElement | undefined = $state();

  // Show fields when selected AND the network is unknown/hidden, is secured
  // without a saved profile, or a prior connection attempt failed.
  let showFields = $derived(
    selected && (forceShowFields || !entry.ssid || (!entry.known && entry.secured)),
  );

  // Reset transient input state when selection toggles; tracks only `selected`.
  $effect(() => {
    const isSelected = selected;
    untrack(() => {
      forceShowFields = false;
      password = "";
      ssid = entry.ssid || "";
      if (!isSelected) transition.clearError();
    });
  });

  // Focus the relevant input whenever the fields become visible.
  $effect(() => {
    if (!showFields) return;
    setTimeout(() => {
      if (ssidInputRef && !ssidInputRef.value) ssidInputRef.focus();
      else passwordInputRef?.focus();
    }, 0);
  });

  async function onConnection() {
    transition.start(async () => {
      if (entry.connected) {
        await invoke(SeelenCommand.WlanDisconnect);
        return;
      }

      // Known networks: try with saved credentials first; on failure ask for password
      if (!showFields && entry.known && entry.ssid) {
        const success = await invoke(SeelenCommand.WlanConnect, {
          ssid: entry.ssid,
          password: null,
          hidden: false,
        });
        forceShowFields = !success;
        return;
      }

      // Unknown / secured networks: user must provide credentials
      const success = await invoke(SeelenCommand.WlanConnect, {
        ssid,
        password: password || null,
        hidden: !entry.ssid,
      });
      if (!success) throw new Error("Connection failed");
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") onConnection();
  }

  async function onForget() {
    if (!entry.ssid) return;
    const ssid = entry.ssid;
    transition.start(async () => {
      await invoke(SeelenCommand.WlanForget, { ssid });
    });
  }

  const signalIcon = $derived.by((): IconName => {
    if (entry.signal > 75) return "GrWifi";
    if (entry.signal > 50) return "GrWifiMedium";
    if (entry.signal > 25) return "GrWifiLow";
    return "GrWifiNone";
  });

  const frequencies = $derived.by(() => {
    const FREQUENCY_BANDS = [
      { name: "2.4G", min: 2_400_000, max: 2_484_000 },
      { name: "5G", min: 5_000_000, max: 5_850_000 },
      { name: "6G", min: 5_925_000, max: 7_125_000 },
    ];
    const detected = new Set<string>();
    for (const e of group) {
      for (const band of FREQUENCY_BANDS) {
        if (e.channelFrequency >= band.min && e.channelFrequency <= band.max) {
          detected.add(band.name);
          break;
        }
      }
    }
    return Array.from(detected);
  });
</script>

<div
  class="wlan-entry"
  class:wlan-entry-selected={selected}
  onclick={() => {
    if (!selected) {
      globalState.selectedSsid = isHiddenGroup ? "__HIDDEN_SSID__" : entry.ssid!;
    }
  }}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      globalState.selectedSsid = isHiddenGroup ? "__HIDDEN_SSID__" : entry.ssid!;
    }
  }}
>
  <div class="wlan-entry-info">
    <Icon iconName={signalIcon} size={20} />
    <span class="wlan-entry-ssid">
      {entry.ssid || `${$t("hidden")} (${group.length})`}
    </span>
    {#if !isHiddenGroup && frequencies.length > 0}
      <div class="wlan-entry-band">{frequencies.join("/")}</div>
    {/if}
    {#if !isHiddenGroup && entry.secured}
      <Icon iconName="PiPasswordFill" title={entry.auth} />
    {/if}
  </div>

  {#if selected}
    {#if showFields && !loading}
      <div class="wlan-entry-fields">
        {#if !entry.ssid}
          <input
            type="text"
            data-skin="default"
            class:error
            bind:value={ssid}
            bind:this={ssidInputRef}
            placeholder="SSID"
          />
        {/if}

        <input
          type="password"
          data-skin="default"
          class:error
          bind:value={password}
          bind:this={passwordInputRef}
          placeholder={$t("password")}
          onkeydown={handleKeydown}
        />
      </div>
    {/if}

    {#if loading}
      <div class="wlan-entry-connecting">{$t("connecting")}</div>
    {:else}
      <div class="wlan-entry-actions">
        <button data-skin={entry.connected ? "default" : "solid"} onclick={onConnection}>
          {entry.connected ? $t("disconnect") : $t("connect")}
        </button>
        {#if entry.known && entry.ssid}
          <button data-skin="default" onclick={onForget}>
            {$t("forget")}
          </button>
        {/if}
      </div>
    {/if}
  {/if}
</div>
