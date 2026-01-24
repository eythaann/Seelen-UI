<script lang="ts">
  import { globalState } from "./state/mod.svelte";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "./i18n";
  import UserProfile from "./components/UserProfile.svelte";
  import UserFolder from "./components/UserFolder.svelte";
  import { path } from "@tauri-apps/api";
  import { knownFolders } from "./state/knownFolders.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  async function openInstallationFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.appDataDir() });
  }

  async function openLogFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.appLogDir() });
  }
</script>

<div class="slu-standard-popover user-popup">
  <UserProfile user={globalState.user} />

  <hr />
  <span class="user-label">{$t("folders.title")}</span>
  {#each Object.entries(knownFolders.value) as folder (folder[0])}
    <UserFolder type={folder[0] as any} {...folder[1]} />
  {/each}

  <hr />
  <span class="user-label">{$t("seelen_options.title")}</span>
  <div class="user-seelen-options">
    <button class="user-seelen-option-item" onclick={openInstallationFolder}>
      <Icon iconName="TbFolderCog" />
      <span class="user-seelen-option-item-title">
        {$t("seelen_options.open_installation_folder")}
      </span>
    </button>
    <button class="user-seelen-option-item" onclick={openLogFolder}>
      <Icon iconName="TbLogs" />
      <span class="user-seelen-option-item-title">
        {$t("seelen_options.open_log_folder")}
      </span>
    </button>
  </div>
</div>
