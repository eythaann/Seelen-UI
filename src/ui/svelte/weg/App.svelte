<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { onMount } from "svelte";
  import { debounce } from "lodash";
  import Dock from "./components/Dock.svelte";
  import { windowsState, focused, widgetStatuses } from "./state/windows.svelte.ts";
  import { settingsState } from "./state/settings.svelte.ts";

  const startMenuExes = ["SearchHost.exe", "StartMenuExperienceHost.exe"];

  const topWindowIsFullscreen = $derived(windowsState.topInteractableWindow?.isFullscreen);

  const focusedIsAppsMenu = $derived(
    startMenuExes.some((program) => (focused.value?.exe || "").endsWith(program)) ||
      widgetStatuses.value.some(
        (w) => w.widgetId === "@seelen/apps-menu" && w.webviewWindowId === focused.value?.hwnd,
      ),
  );

  const showWidget = $derived(!topWindowIsFullscreen || focusedIsAppsMenu);
  const setWidgetVisibility = debounce((value: boolean) => {
    if (value) {
      Widget.self.show();
    } else {
      Widget.self.hide();
    }
  }, 100);

  $effect(() => {
    if (settingsState.isReady) {
      setWidgetVisibility(showWidget);
    }
  });

  onMount(() => {
    Widget.self.ready({ show: false }).then(() => {
      settingsState.isReady = true;
    });
  });
</script>

<Dock />
