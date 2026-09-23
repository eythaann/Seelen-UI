<script lang="ts">
  import { globalState } from "./state.svelte";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { NotificationsMode } from "@seelen-ui/lib/types";
  import { t } from "./i18n/index.ts";
  import Notification from "./components/Notification.svelte";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { notificationCardExit } from "./transitions.ts";
  import { fade } from "svelte/transition";

  let exitingCount = $state(0);
  let isInitialLoad = $state(true);

  $effect(() => {
    Widget.getCurrent().ready();
    const timer = setTimeout(() => {
      isInitialLoad = false;
    }, 400);
    return () => clearTimeout(timer);
  });

  async function handleClearAll() {
    try {
      await invoke(SeelenCommand.NotificationsCloseAll);
    } catch (error) {
      console.error("Failed to clear notifications:", error);
    }
  }

  const isDndActive = $derived(globalState.focusAssistMode !== NotificationsMode.All);
  async function toggleDnd() {
    await invoke(SeelenCommand.SetNotificationsMode, {
      mode: isDndActive ? NotificationsMode.All : NotificationsMode.AlarmsOnly,
    });
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

<div class="slu-std-popover notifications-popup">
  <div class="notifications-popup-header">
    <span>{$t("title")}</span>
    <button
      data-skin={isDndActive ? "solid" : "default"}
      onclick={toggleDnd}
      aria-label={$t("dnd")}
    >
      <Icon iconName={isDndActive ? "IoMoon" : "IoMoonOutline"} />
    </button>
    <button
      data-skin="default"
      onclick={handleClearAll}
      disabled={globalState.notifications.length === 0 || exitingCount > 0}
    >
      {$t("clear")}
    </button>
  </div>

  <div class="notifications-popup-body">
    {#each globalState.notifications as notification (notification.id)}
      <div
        class="notification-card-container"
        out:notificationCardExit|global
        onoutrostart={() => {
          exitingCount++;
        }}
        onoutroend={() => {
          exitingCount--;
        }}
      >
        <Notification {notification} />
      </div>
    {/each}

    {#if globalState.notifications.length === 0 && exitingCount === 0}
      <div
        class="notifications-popup-empty"
        in:fade={{ duration: isInitialLoad ? 0 : 180 }}
      >
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
  .notification-card-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    min-width: 0;
  }
</style>
