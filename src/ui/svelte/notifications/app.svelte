<script lang="ts">
  import { globalState } from "./state.svelte";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { NotificationsMode } from "@seelen-ui/lib/types";
  import { t } from "./i18n/index.ts";
  import Notification from "./components/Notification.svelte";
  import { Icon } from "libs/ui/svelte/components/Icon";

  let isClearing = $state(false);

  $effect(() => {
    Widget.getCurrent().ready();
  });

  async function handleClearAll() {
    if (isClearing) {
      return;
    }
    isClearing = true;
    try {
      await invoke(SeelenCommand.NotificationsCloseAll);
    } catch (error) {
      console.error("Failed to clear notifications:", error);
    } finally {
      isClearing = false;
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
      disabled={globalState.notifications.length === 0 || isClearing}
    >
      {$t("clear")}
    </button>
  </div>

  <div class="notifications-popup-body">
    <div class="notifications-layout-stack">
      <div class="notifications-cards-layer">
        {#each globalState.notifications as notification (notification.id)}
          <Notification {notification} />
        {/each}
      </div>

      {#if globalState.notifications.length === 0}
        <div class="notifications-popup-empty">
          <p>{$t("empty")}</p>
        </div>
      {/if}
    </div>
  </div>

  <div class="notifications-popup-footer">
    <button data-skin="transparent" onclick={handleOpenSettings}>
      {$t("settings")}
    </button>
  </div>
</div>

<style>
  .notifications-layout-stack {
    display: grid;
    grid-template-columns: 100%;
    grid-template-rows: auto;
    width: 100%;
    min-width: 0;
    gap: inherit;
  }

  .notifications-cards-layer {
    grid-area: 1 / 1;
    display: flex;
    flex-direction: column;
    gap: inherit;
    width: 100%;
    min-width: 0;
    z-index: 1;
  }

  .notifications-cards-layer:empty {
    pointer-events: none;
  }

  .notifications-popup-empty {
    grid-area: 1 / 1;
    z-index: 0;
  }
</style>
