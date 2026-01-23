<script lang="ts">
  import type { WlanBssEntry } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
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
        (isHiddenGroup && globalState.selectedSsid === "__HIDDEN_SSID__"))
  );

  let loading = $derived(transition.loading);
  let error = $derived(transition.error);

  let showFields = $state(false);
  let ssid = $state("");
  let password = $state("");
  let ssidInputRef: HTMLInputElement | undefined = $state();
  let passwordInputRef: HTMLInputElement | undefined = $state();

  // Reset state when selection changes
  $effect(() => {
    if (!selected) {
      showFields = false;
      ssid = entry.ssid || "";
      password = "";
      transition.clearError();
    } else {
      showFields = !entry.known && (!entry.ssid || entry.secured);
      ssid = entry.ssid || "";
      // Focus inputs when shown
      if (showFields) {
        setTimeout(() => {
          if (!entry.ssid && ssidInputRef) {
            ssidInputRef.focus();
          } else if (passwordInputRef) {
            passwordInputRef.focus();
          }
        }, 0);
      }
    }
  });

  async function onConnection() {
    transition.start(async () => {
      if (entry.connected) {
        await invoke(SeelenCommand.WlanDisconnect);
        return;
      }

      if (showFields) {
        const success = await invoke(SeelenCommand.WlanConnect, {
          ssid,
          password,
          hidden: !entry.ssid,
        });
        if (!success) {
          throw new Error("Connection failed");
        }
        return;
      }

      const profiles = await invoke(SeelenCommand.WlanGetProfiles);
      const profile = profiles.find((p) => p.ssid === entry.ssid);

      if (!profile) {
        showFields = true;
        return;
      }

      const success = await invoke(SeelenCommand.WlanConnect, {
        ssid: profile.ssid,
        password: profile.password,
        hidden: !entry.ssid,
      });
      if (!success) {
        throw new Error("Connection failed");
      }
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      onConnection();
    }
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

    const detectedBands = new Set<string>();
    for (const e of group) {
      for (const band of FREQUENCY_BANDS) {
        if (e.channelFrequency >= band.min && e.channelFrequency <= band.max) {
          detectedBands.add(band.name);
          break;
        }
      }
    }
    return Array.from(detectedBands);
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
      <Icon iconName="PiPasswordFill" title={$t("secured")} />
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

    {#if !loading}
      <div class="wlan-entry-actions">
        <button
          data-skin={entry.connected ? "default" : "solid"}
          onclick={onConnection}
          disabled={loading}
        >
          {entry.connected ? $t("disconnect") : $t("connect")}
        </button>
      </div>
    {/if}
  {/if}
</div>
