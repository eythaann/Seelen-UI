<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { SeelenWegSide, WegMiddleClickAction, type UserAppWindow } from "@seelen-ui/lib/types";
  import { FileIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { t } from "../../i18n/index.ts";
  import type { AppOrFileWegItem } from "../../types.ts";
  import { settingsState, getDockContextMenuAlignment } from "../../state/settings.svelte.ts";
  import { windowsState, focused } from "../../state/windows.svelte.ts";
  import { notifications } from "../../state/system.svelte.ts";
  import { isTouchPrimary } from "libs/ui/svelte/utils";
  import { getUserApplicationContextMenu, launchItem } from "../../appMenu.ts";
  import UserApplicationPreview from "./UserApplicationPreview.svelte";

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

  // Preview popup state
  let showPreview = $state(false);
  let previewX = $state(0);
  let previewY = $state(0);
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;
  let itemEl: HTMLDivElement | null = $state(null);

  function computePreviewPosition() {
    if (!itemEl) return;
    const rect = itemEl.getBoundingClientRect();
    const pos = settingsState.position;

    switch (pos) {
      case SeelenWegSide.Bottom:
        previewX = rect.left + rect.width / 2;
        previewY = rect.top;
        break;
      case SeelenWegSide.Top:
        previewX = rect.left + rect.width / 2;
        previewY = rect.bottom;
        break;
      case SeelenWegSide.Left:
        previewX = rect.right;
        previewY = rect.top + rect.height / 2;
        break;
      case SeelenWegSide.Right:
        previewX = rect.left;
        previewY = rect.top + rect.height / 2;
        break;
    }
  }

  function onMouseEnter() {
    if (isOverlay || windows.length === 0 || isTouchPrimary.value) return;
    hoverTimeout = setTimeout(() => {
      computePreviewPosition();
      showPreview = true;
    }, 300);
  }

  function onMouseLeave() {
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
      hoverTimeout = null;
    }
    showPreview = false;
  }

  function onClick() {
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
    const { alignX, alignY } = getDockContextMenuAlignment(settingsState.position);
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getUserApplicationContextMenu($t, item, windows), alignX, alignY },
      forwardTo: null,
    });
  }

  function getPreviewContainerClass() {
    return `weg-item-preview-container ${settingsState.position.toLowerCase()}`;
  }

  function getPreviewStyle() {
    const pos = settingsState.position;
    switch (pos) {
      case SeelenWegSide.Bottom:
        return `left: ${previewX}px; bottom: ${window.innerHeight - previewY}px; transform: translateX(-50%);`;
      case SeelenWegSide.Top:
        return `left: ${previewX}px; top: ${previewY}px; transform: translateX(-50%);`;
      case SeelenWegSide.Left:
        return `left: ${previewX}px; top: ${previewY}px; transform: translateY(-50%);`;
      case SeelenWegSide.Right:
        return `right: ${window.innerWidth - previewX}px; top: ${previewY}px; transform: translateY(-50%);`;
    }
  }
</script>

<div
  bind:this={itemEl}
  role="menu"
  tabindex="0"
  class="weg-item-overlay"
  onmouseenter={onMouseEnter}
  onmouseleave={onMouseLeave}
>
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

{#if showPreview && windows.length > 0 && !isOverlay && !isTouchPrimary.value}
  <div class={getPreviewContainerClass()} style={getPreviewStyle()}>
    <div class="weg-item-preview-scrollbar">
      {#each windows as win (win.hwnd)}
        <UserApplicationPreview title={win.title} hwnd={win.hwnd} />
      {/each}
    </div>
  </div>
{/if}
