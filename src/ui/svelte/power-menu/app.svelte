<script lang="ts">
  import { onMount } from "svelte";
  import { options } from "./options";
  import { state as globalState } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { Widget } from "@seelen-ui/lib";
  import { t } from "./i18n";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";
  import { monitors } from "libs/ui/svelte/runes/monitors.svelte";

  onMount(() => {
    Widget.getCurrent().ready();
  });

  let installUpdates = $state(true);
  const willInstallUpdates = $derived(globalState.hasPendingOsUpdates && installUpdates);

  function onCancel() {
    Widget.self.hide();
  }

  const menu = $derived.by(() => {
    if (!monitors.primaryMonitor) {
      return null;
    }

    const { rect, scaleFactor } = monitors.primaryMonitor;
    const relativeRect = monitors.calculateDesktopRelativeRect(rect);

    return {
      x: relativeRect.left,
      y: relativeRect.top,
      // we reduce the size and later scale it, to get the correct display by dpi aware
      width: (rect.right - rect.left) / scaleFactor,
      height: (rect.bottom - rect.top) / scaleFactor,
      scale: scaleFactor,
    };
  });
</script>

<svelte:window
  onclick={onCancel}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      onCancel();
    }
  }}
/>

{#if menu}
  <div
    class="power-menu-monitor"
    style:position="fixed"
    style:left={menu.x + "px"}
    style:top={menu.y + "px"}
    style:width={menu.width + "px"}
    style:height={menu.height + "px"}
    style:transform={`scale(${menu.scale})`}
    style:transform-origin="left top"
  >
    <div
      class="power-popup"
      role="dialog"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={() => {}}
    >
      <div class="power-menu-header">
        <div class="power-menu-user">
          {#if globalState.user.profilePicturePath}
            <img
              class="power-menu-user-profile"
              src={convertFileSrc(globalState.user.profilePicturePath)}
              alt=""
            />
          {:else}
            <MissingIcon class="power-menu-user-profile" />
          {/if}
          <div class="power-menu-user-info">
            <div class="power-menu-bye-bye">{$t("goodbye", { 0: globalState.user.name })}</div>
            <div class="power-menu-user-email">{globalState.user.email}</div>
          </div>
        </div>
      </div>

      {#if globalState.hasPendingOsUpdates}
        <label class="power-menu-updates">
          <span class="power-menu-updates-message">{$t("pending_updates")}</span>
          <input type="checkbox" data-skin="switch" bind:checked={installUpdates} />
        </label>
      {/if}

      <div class="power-menu-separator"></div>

      <ul class="power-menu-list">
        {#each options as option}
          {@const showUpdate = willInstallUpdates && !!option.updateKey}
          <li>
            <button
              onclick={() => {
                option.onClick(willInstallUpdates);
                onCancel();
              }}
              class="power-menu-item"
            >
              <div class="power-menu-item-icon-wrapper">
                <Icon iconName={option.icon as any} />
                {#if showUpdate}
                  <span class="power-menu-item-update-badge"></span>
                {/if}
              </div>
              <span class="power-menu-item-label">{$t(showUpdate ? option.updateKey! : option.key)}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </div>
{/if}
