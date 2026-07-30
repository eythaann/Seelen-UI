<script lang="ts">
  import { t } from "../../i18n";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import type { WidgetId } from "@seelen-ui/lib/types";
  import { globalState } from "../../state/mod.svelte";
  import { StartDisplayMode } from "../../constants";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { convertFileSrc } from "@tauri-apps/api/core";

  function openUserMenu() {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/user-menu" as WidgetId },
    });
  }

  function openAppSettings() {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/settings" as WidgetId },
    });
  }

  function openPowerMenu() {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/power-menu" as WidgetId },
    });
  }
</script>

<div class="apps-menu-footer">
  <div class="apps-menu-footer-left">
    <button data-skin="transparent" class="user-profile" onclick={openUserMenu}>
      {#if globalState.user.profilePicturePath}
        <img
          class="user-profile-picture"
          src={convertFileSrc(globalState.user.profilePicturePath)}
          alt={globalState.user.name}
        />
      {:else}
        <Icon class="user-profile-picture" iconName="PiFolderUser" />
      {/if}

      <span>{globalState.user.name}</span>
    </button>
  </div>

  <div class="apps-menu-footer-right">
    <button data-skin="transparent" onclick={openAppSettings} title={$t("app_settings")}>
      <Icon iconName="RiSettings4Fill" />
    </button>

    <button data-skin="transparent" onclick={openPowerMenu} title={$t("power_menu")}>
      <Icon iconName="IoPower" />
    </button>

    <button
      data-skin="transparent"
      onclick={() => {
        globalState.displayMode =
          globalState.displayMode === StartDisplayMode.Normal
            ? StartDisplayMode.Fullscreen
            : StartDisplayMode.Normal;
      }}
    >
      <Icon
        iconName={globalState.displayMode === StartDisplayMode.Fullscreen
          ? "IoContract"
          : "IoExpand"}
      />
    </button>
  </div>
</div>
