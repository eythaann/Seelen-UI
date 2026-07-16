<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { WegMiddleClickAction, type UserAppWindow } from "@seelen-ui/lib/types";
  import { FileIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { t } from "../../i18n/index.ts";
  import type { AppOrFileWegItem } from "../../types.ts";
  import { settingsState } from "../../state/settings.svelte.ts";
  import { windowsState, focused } from "../../state/windows.svelte.ts";
  import { notifications } from "../../state/getters.svelte.ts";
  import { getUserApplicationContextMenu, launchItem } from "../../appMenu.ts";
  import { triggerPreviewWidget } from "../../previewWidget.ts";

  interface Props {
    item: AppOrFileWegItem;
    windows: UserAppWindow[];
    isOverlay?: boolean;
  }

  let { item, windows, isOverlay = false }: Props = $props();

  const settings = $derived(settingsState.value as any);
  const notificationsCount = $derived(
    notifications.value.filter((n: any) => n.appUmid === item.umid).length,
  );
  const itemLabel = $derived(
    settings?.showWindowTitle && windows.length ? windows[0]!.title : null,
  );
  const isFocused = $derived(windows.some((w) => w.hwnd === focused.value?.hwnd));

  let itemEl: HTMLDivElement | null = $state(null);

  function onClick() {
    if (windows.length > 1) {
      triggerPreviewWidget(itemEl!, windows);
      return;
    }

    const win = windows[0];
    if (!win) {
      launchItem(item, false);
    } else {
      invoke(SeelenCommand.WegToggleWindowState, {
        hwnd: win.hwnd,
        wasFocused: windowsState.delayedFocused?.hwnd === win.hwnd,
      });
    }
  }

  function onAuxClick(e: MouseEvent) {
    if (e.button !== 1) return;
    if (settings?.middleClickAction === WegMiddleClickAction.OpenNewInstance) {
      launchItem(item, false);
    } else {
      const win = windows[0];
      if (win) invoke(SeelenCommand.WegCloseApp, { hwnd: win.hwnd });
    }
  }

  function onContextMenu(e: MouseEvent) {
    e.stopPropagation();
    const alignX = settingsState.popupAlignX;
    const alignY = settingsState.popupAlignY;
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getUserApplicationContextMenu($t, item, windows), alignX, alignY },
      forwardTo: null,
    });
  }
</script>

<div bind:this={itemEl} role="menu" tabindex="0" class="weg-item-overlay">
  <div
    role="menuitem"
    tabindex="0"
    class="weg-item"
    onclick={onClick}
    onauxclick={onAuxClick}
    oncontextmenu={onContextMenu}
    onkeypress={() => {}}
  >
    <FileIcon class="weg-item-icon" path={item.relaunch?.icon || item.path} umid={item.umid} />
    {#if itemLabel}
      <div class="weg-item-title">{itemLabel}</div>
    {/if}
  </div>

  {#if notificationsCount > 0}
    <div class="weg-item-notification-badge">{notificationsCount}</div>
  {/if}

  {#if settings?.showInstanceCounter && windows.length > 1}
    <div class="weg-item-instance-counter-badge">{windows.length}</div>
  {/if}

  {#if !settings?.showWindowTitle}
    <div
      class="weg-item-open-sign"
      class:weg-item-open-sign-active={windows.length > 0}
      class:weg-item-open-sign-focused={isFocused}
    ></div>
  {/if}
</div>
