<script lang="ts">
  import { t } from "./i18n";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import StartMenuBody from "./components/layout/StartMenuBody.svelte";
  import StartMenuFooter from "./components/layout/StartMenuFooter.svelte";
  import { globalState } from "./state/mod.svelte";
  import { searchState } from "./state/search.svelte";
  import { StartDisplayMode, StartView } from "./constants";
  import { createInputKeyDownHandler } from "./keyboard-navigation";

  let inputElement: HTMLInputElement;

  // Detect and manage query prefix
  const queryPrefix = $derived.by(() => {
    const query = searchState.searchQuery.trim();
    const match = query.match(/^(apps|files|web):/i);
    return match ? match[1]?.toLowerCase() || "" : "";
  });

  function handlePrefixChange(newPrefix: string) {
    const query = searchState.searchQuery.trim();
    const match = query.match(/^(apps|files|web):/i);
    const search = match ? query.slice(match[0].length).trim() : query;

    if (newPrefix) {
      searchState.searchQuery = `${newPrefix}:${search ? " " + search : ""}`;
    } else {
      searchState.searchQuery = search;
    }

    inputElement.focus();
  }

  function handleDocKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      Widget.self.hide();
    }
  }

  const handleInputKeyDown = createInputKeyDownHandler({
    onEnter: (_event) => {
      // Check for web: prefix
      const query = searchState.searchQuery.trim();
      const webPrefixMatch = query.match(/^web:/i);

      if (webPrefixMatch) {
        Widget.self.hide();
        const searchQuery = query.slice(4).trim();
        const encodedQuery = encodeURIComponent(searchQuery);
        invoke(SeelenCommand.OpenFile, {
          path: `https://www.google.com/search?q=${encodedQuery}`,
        });
        return true;
      }

      return false;
    },
  });

  // reset state when menu is shown
  $effect(() => {
    if (globalState.version) {
      searchState.searchQuery = "";
      inputElement.focus();
    }
  });

  // Searching always shows results in the "all apps" view
  $effect(() => {
    if (searchState.searchQuery) {
      globalState.view = StartView.All;
    }
  });

  $effect(() => {
    if (!Widget.self.isReady) {
      void Widget.self.ready({ show: false });
    }
  });
</script>

<svelte:window onkeydown={handleDocKeyDown} />
<div class="apps-menu" data-fullscreen={globalState.displayMode === StartDisplayMode.Fullscreen}>
  <div class="apps-menu-header">
    <input
      bind:this={inputElement}
      bind:value={searchState.searchQuery}
      type="search"
      data-skin="transparent"
      placeholder={$t("applications")}
      onkeydown={handleInputKeyDown}
    />

    {#if searchState.searchQuery}
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
        searchState.searchQuery = "";
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
  <StartMenuFooter />
</div>
