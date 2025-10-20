<script>
  import { onMount } from "svelte";
  import { options } from "./options";
  import { setup } from "./actions";
  import { state } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { Widget } from "@seelen-ui/lib";
  import { t } from "./i18n";

  onMount(() => {
    setup(state);
  });

  function onCancel() {
    Widget.getCurrent().webview.hide();
  }
</script>

<div
  class="power-menu"
  role="menu"
  tabindex="-1"
  on:click={onCancel}
  on:keydown={(e) => {
    if (e.key === "Escape") {
      onCancel();
    }
  }}
>
  <div class="power-menu-user">
    <img
      class="power-menu-user-profile"
      src={convertFileSrc(state.user.profilePicturePath)}
      alt=""
    />
    <div class="power-menu-user-email">
      {state.user.email}
    </div>
  </div>
  <div class="power-menu-bye-bye">{$t("goodbye", [state.user.name])}</div>
  <ul class="power-menu-list">
    {#each options as option}
      <li>
        <button on:click={option.action} class="power-menu-item">
          <Icon iconName={option.icon} />
          <span class="power-menu-item-label">{$t(option.key)}</span>
        </button>
      </li>
    {/each}
  </ul>
  <!-- <div class="power-menu-uptime">{$t("uptime")}: 2 hours 30 minutes</div> -->
</div>

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }
</style>
