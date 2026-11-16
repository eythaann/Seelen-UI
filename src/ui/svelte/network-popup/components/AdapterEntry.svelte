<script lang="ts">
  import type { NetworkAdapter } from "@seelen-ui/lib/types";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";

  interface Props {
    adapter: NetworkAdapter;
  }

  let { adapter }: Props = $props();

  // Link up is not enough: without gateway (or with a self-assigned APIPA address, DHCP failed)
  // the cable is plugged but there is no way out. The default route also needs real internet.
  const isApipa = $derived(!!adapter.ipv4?.startsWith("169.254."));
  const hasAccess = $derived(
    adapter.isDefault ? globalState.online : !!adapter.gateway && !isApipa,
  );
  const ip = $derived(adapter.ipv4 ?? adapter.ipv6);

  let copied = $state(false);
  let copiedTimeout: ReturnType<typeof setTimeout> | undefined;

  async function copyIp() {
    if (!ip) return;
    await navigator.clipboard.writeText(ip);
    copied = true;
    clearTimeout(copiedTimeout);
    copiedTimeout = setTimeout(() => (copied = false), 1500);
  }
</script>

<div class="adapter-entry">
  <Icon
    iconName={hasAccess ? "TbNetwork" : "TbNetworkOff"}
    title={adapter.isPhysical
      ? adapter.name
      : `${$t("adapter.virtual")} · ${adapter.name}`}
  />

  <div class="adapter-entry-info">
    <span class="adapter-entry-header">
      <span class="adapter-entry-name">{adapter.description}</span>
    </span>

    <span class="adapter-entry-details">
      <span class:adapter-entry-no-access={!hasAccess}>
        {hasAccess ? $t("adapter.connected") : $t("adapter.no_internet")}
      </span>
      {#if ip}
        <span class="adapter-entry-ip">{ip}</span>
        <button
          class="adapter-entry-copy"
          data-skin="transparent"
          title={$t("adapter.copy_ip")}
          onclick={copyIp}
        >
          <Icon iconName={copied ? "IoCheckmark" : "IoCopyOutline"} />
        </button>
      {/if}
    </span>
  </div>
</div>
