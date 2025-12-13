<script lang="ts">
  import { onMount } from "svelte";
  import { options } from "./options";
  import { setup } from "./actions";
  import { state } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { Widget } from "@seelen-ui/lib";
  import { t } from "./i18n";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";

  onMount(() => {
    setup(state);
    Widget.getCurrent().ready();
  });

  function onCancel() {
    Widget.getCurrent().webview.hide();
  }

  const width = $derived(
    (state.menuRect.right - state.menuRect.left) / globalThis.devicePixelRatio
  );
  const height = $derived(
    (state.menuRect.bottom - state.menuRect.top) / globalThis.devicePixelRatio
  );
</script>

<div
  class="power-menu-overlay"
  role="menu"
  tabindex="-1"
  onclick={onCancel}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      onCancel();
    }
  }}
>
  <div
    class="power-menu"
    style:position="fixed"
    style:left="{state.menuRect.left / globalThis.devicePixelRatio}px"
    style:top="{state.menuRect.top / globalThis.devicePixelRatio}px"
    style:width="{width}px"
    style:height="{height}px"
  >
    <div class="power-menu-user">
      {#if state.user.profilePicturePath}
        <img
          class="power-menu-user-profile"
          src={convertFileSrc(state.user.profilePicturePath)}
          alt=""
        />
      {:else}
        <MissingIcon class="power-menu-user-profile" />
      {/if}
      <div class="power-menu-user-email">
        {state.user.email}
      </div>
    </div>
    <div class="power-menu-bye-bye">{$t("goodbye", { 0: state.user.name })}</div>
    <ul class="power-menu-list">
      {#each options as option}
        <li>
          <button onclick={option.onClick} class="power-menu-item">
            <Icon iconName={option.icon as any} />
            <span class="power-menu-item-label">{$t(option.key)}</span>
          </button>
        </li>
      {/each}
    </ul>
    <!-- <div class="power-menu-uptime">{$t("uptime")}: 2 hours 30 minutes</div> -->
  </div>
</div>

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }
</style>
