<script lang="ts">
  import type { User } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { path } from "@tauri-apps/api";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "../i18n";

  interface Props {
    user: User;
  }

  let { user }: Props = $props();

  async function openUserFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.homeDir() });
  }

  function openOneDrive() {
    if (user.oneDrivePath) {
      invoke(SeelenCommand.OpenFile, { path: user.oneDrivePath });
    }
  }

  function openAccountSettings() {
    invoke(SeelenCommand.OpenFile, { path: "ms-settings:accounts" });
  }

  function logOut() {
    invoke(SeelenCommand.LogOut);
  }
</script>

<div class="user-profile-container">
  <div class="user-profile-picture-container">
    {#if user.profilePicturePath}
      <img
        class="user-profile-picture"
        src={convertFileSrc(user.profilePicturePath)}
        alt={user.name}
      />
    {:else}
      <div class="user-profile-picture-fallback">
        <Icon iconName="PiFolderUser" />
      </div>
    {/if}
    <button
      class="user-profile-lock-button"
      onclick={logOut}
      title={$t("profile.log_out")}
    >
      <Icon iconName="BiLogOut" />
    </button>
    <button
      class="user-profile-settings-button"
      onclick={openAccountSettings}
      title={$t("profile.accounts")}
    >
      <Icon iconName="RiSettings3Fill" />
    </button>
  </div>

  <div class="user-profile-information">
    <div class="user-profile-name">
      <span>{user.name}</span>
      <button
        class="user-profile-action-button"
        onclick={openUserFolder}
        title={$t("profile.open_user_folder")}
      >
        <Icon iconName="PiFolderUser" />
      </button>
    </div>

    {#if user.email}
      <div class="user-profile-email">{user.email}</div>
    {/if}

    {#if user.oneDrivePath}
      <button
        class="user-profile-action-button"
        onclick={openOneDrive}
        title={$t("profile.open_onedrive")}
      >
        <Icon iconName="ImOnedrive" />
        <span>OneDrive</span>
      </button>
    {/if}
  </div>
</div>
