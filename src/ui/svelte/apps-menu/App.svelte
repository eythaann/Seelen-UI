<script lang="ts">
  import { t } from "./i18n";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import type { WidgetId } from "@seelen-ui/lib/types";
  import StartMenuBody from "./components/StartMenuBody.svelte";
  import { globalState } from "./state/mod.svelte";
  import { StartDisplayMode, StartView } from "./constants";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { navigateInDirection } from "./keyboard-navigation";
  import { convertFileSrc } from "@tauri-apps/api/core";

  let inputElement: HTMLInputElement | undefined = $state();

  // Detect and manage query prefix
  const queryPrefix = $derived.by(() => {
    const query = globalState.searchQuery.trim();
    const match = query.match(/^(apps|files|web):/i);
    return match ? match[1]?.toLowerCase() || "" : "";
  });

  function handlePrefixChange(newPrefix: string) {
    const query = globalState.searchQuery.trim();
    const match = query.match(/^(apps|files|web):/i);
    const search = match ? query.slice(match[0].length).trim() : query;

    if (newPrefix) {
      globalState.searchQuery = `${newPrefix}:${search ? " " + search : ""}`;
    } else {
      globalState.searchQuery = search;
    }

    inputElement?.focus();
  }

  function handleDocKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case "Escape":
        event.preventDefault();
        globalState.showing = false;
        break;
      case "ArrowUp":
        event.preventDefault();
        navigateInDirection("up");
        break;
      case "ArrowDown":
        event.preventDefault();
        navigateInDirection("down");
        break;
      case "ArrowLeft":
        event.preventDefault();
        navigateInDirection("left");
        break;
      case "ArrowRight":
        event.preventDefault();
        navigateInDirection("right");
        break;
    }
  }

  function handleInputKeyDown(event: KeyboardEvent) {
    const input = event.currentTarget as HTMLInputElement;

    switch (event.key) {
      case "Enter":
        event.preventDefault();

        // Check for web: prefix
        const query = globalState.searchQuery.trim();
        const webPrefixMatch = query.match(/^web:/i);

        if (webPrefixMatch) {
          const searchQuery = query.slice(4).trim();
          globalState.showing = false;
          const encodedQuery = encodeURIComponent(searchQuery);
          invoke(SeelenCommand.OpenFile, {
            path: `https://www.google.com/search?q=${encodedQuery}`,
          });
          break;
        }

        // Click on preselected item or first item if none selected
        let element: HTMLElement | null = null;

        if (globalState.preselectedItem) {
          element = document.querySelector(`[data-item-id="${globalState.preselectedItem}"]`);
        } else {
          element = document.querySelector(".app");
        }

        if (element) {
          element.click();
        }
        break;
      case "ArrowDown":
        event.stopPropagation();
        navigateInDirection("down");
        break;
      case "ArrowRight":
        event.stopPropagation();
        // Only navigate if cursor is at the end of the input
        if (input.selectionStart === input.value.length) {
          navigateInDirection("right");
        }
        break;
    }
  }

  // reset state when menu is shown
  $effect(() => {
    if (globalState.showing) {
      globalState.searchQuery = "";
      globalState.preselectedItem = null;
      inputElement?.focus();
    }
  });

  // Reset preselected item when search query changes
  $effect(() => {
    if (globalState.searchQuery) {
      globalState.view = StartView.All;
    }
    globalState.preselectedItem = null;
  });

  $effect(() => {
    Widget.self.ready({ show: false });
  });

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

<svelte:window onkeydown={handleDocKeyDown} />
<div class="apps-menu" class:fullscreen={globalState.displayMode === StartDisplayMode.Fullscreen}>
  <div class="apps-menu-header">
    <input
      bind:this={inputElement}
      bind:value={globalState.searchQuery}
      type="search"
      data-skin="transparent"
      placeholder="Applications"
      onkeydown={handleInputKeyDown}
    />

    {#if globalState.searchQuery}
      <select
        data-skin="default"
        value={queryPrefix}
        onchange={(e) => handlePrefixChange(e.currentTarget.value)}
      >
        <option value="">{$t("query.all")}</option>
        <option value="apps">{$t("query.apps")}</option>
        <option value="files">{$t("query.files")}</option>
        <option value="web">{$t("query.web")}</option>
      </select>
    {/if}

    <button
      data-skin="default"
      onclick={() => {
        globalState.view =
          globalState.view === StartView.Favorites ? StartView.All : StartView.Favorites;
        globalState.searchQuery = "";
      }}
    >
      {#if globalState.view === StartView.Favorites}
        {$t("all")}
      {:else}
        {$t("back")}
      {/if}
    </button>
  </div>

  <StartMenuBody />

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
      <button data-skin="transparent" onclick={openAppSettings} title="App Settings">
        <Icon iconName="RiSettings4Fill" />
      </button>

      <button data-skin="transparent" onclick={openPowerMenu} title="Power Menu">
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
</div>
