<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { options } from "./options";
  import { state as globalState } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { Widget } from "@seelen-ui/lib";
  import { t } from "./i18n";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";

  let isVisible = $state(false);
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    Widget.getCurrent().ready();
    isVisible = true;

    window.addEventListener("focus", () => {
      if (hideTimeout) {
        clearTimeout(hideTimeout);
        hideTimeout = null;
      }
      globalState.refreshUser();
      isVisible = true;
    });
  });

  function onCancel() {
    isVisible = false;
    hideTimeout = setTimeout(() => {
      Widget.self.hide();
    }, 200);
  }

  const menu = $derived.by(() => {
    if (!globalState.primaryMonitor) {
      return null;
    }

    const {
      primaryMonitor: { rect, scaleFactor },
    } = globalState;

    return {
      x: rect.left,
      y: rect.top,
      // we reduce the size and later scale it, to get the correct display by dpi aware
      width: (rect.right - rect.left) / scaleFactor,
      height: (rect.bottom - rect.top) / scaleFactor,
      scale: scaleFactor,
    };
  });
</script>

<svelte:window onkeydown={(e) => {
  if (e.key === "Escape") {
    onCancel();
  }
}} />

{#if isVisible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="power-menu-overlay"
    role="menu"
    tabindex="-1"
    onclick={onCancel}
    transition:fade={{ duration: 200 }}
  >
    {#if menu}
      <div
        class="power-menu"
        style:position="fixed"
        style:left={menu.x + "px"}
        style:top={menu.y + "px"}
        style:width={menu.width + "px"}
        style:height={menu.height + "px"}
        style:transform={`scale(${menu.scale})`}
        style:transform-origin="left top"
      >
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="power-popup" onclick={(e) => e.stopPropagation()}>
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
          
          <div class="power-menu-separator"></div>

          <ul class="power-menu-list">
            {#each options as option}
              <li>
                <button onclick={() => { option.onClick(); onCancel(); }} class="power-menu-item">
                  <div class="power-menu-item-icon-wrapper">
                    <Icon iconName={option.icon as any} />
                  </div>
                  <span class="power-menu-item-label">{$t(option.key)}</span>
                </button>
              </li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }
</style>
