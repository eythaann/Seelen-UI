<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "./i18n";
  import { Widget } from "@seelen-ui/lib";
  import PinnedView from "./components/PinnedView.svelte";
  import AllAppsView from "./components/AllAppsView.svelte";
  import { globalState } from "./state.svelte";
  import { StartDisplayMode, StartView } from "./constants";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { navigateInDirection } from "./keyboard-navigation";

  let inputElement: HTMLInputElement | undefined = $state();

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
        // Click on preselected item or first item if none selected
        let element: HTMLElement | null = null;

        if (globalState.preselectedItem) {
          element = document.querySelector(
            `[data-item-id="${globalState.preselectedItem}"]`
          ) as HTMLElement;
        } else {
          element = document.querySelector(".app-item") as HTMLElement;
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

  onMount(() => {
    Widget.getCurrent().ready();
  });
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

  <div class="apps-menu-body">
    {#if globalState.view === StartView.Favorites}
      <PinnedView />
    {:else if globalState.view === StartView.All}
      <AllAppsView />
    {/if}
  </div>

  <div class="apps-menu-footer">
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
