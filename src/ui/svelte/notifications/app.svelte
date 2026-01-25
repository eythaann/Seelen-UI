<script lang="ts">
  import { globalState } from "./state.svelte";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { t } from "./i18n/index.ts";
  import Notification from "./components/Notification.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  async function handleClearAll() {
    try {
      await invoke(SeelenCommand.NotificationsCloseAll);
    } catch (error) {
      console.error("Failed to clear notifications:", error);
    }
  }

  async function handleOpenSettings() {
    try {
      await invoke(SeelenCommand.OpenFile, {
        path: "ms-settings:notifications",
      });
    } catch (error) {
      console.error("Failed to open notification settings:", error);
    }
  }
</script>

<div class="slu-standard-popover notifications-popup">
  <div class="notifications-popup-header">
    <span>{$t("title")}</span>
    <button data-skin="default" onclick={handleClearAll}>
      {$t("clear")}
    </button>
  </div>

  <div class="notifications-popup-body">
    {#each globalState.notifications as notification (notification.id)}
      <Notification {notification} />
    {/each}

    {#if globalState.notifications.length === 0}
      <div class="notifications-popup-empty">
        <p>{$t("empty")}</p>
      </div>
    {/if}
  </div>

  <div class="notifications-popup-footer">
    <button data-skin="transparent" onclick={handleOpenSettings}>
      {$t("settings")}
    </button>
  </div>
</div>

<style>
  .notifications-popup {
    background: var(--config-background-color, var(--color-gray-100));
    color: var(--config-foreground-color, var(--color-gray-900));
    border-radius: 8px;
    overflow: hidden;
  }
</style>
