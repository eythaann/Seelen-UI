<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { ZOrder } from "@seelen-ui/lib/types";
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
        (w) =>
          w.widgetId === "@seelen/apps-menu" && w.webviewWindowId === focused.value?.hwnd,
      ),
  );

  const alwaysOnTop = $derived(!topWindowIsFullscreen || focusedIsAppsMenu);

  const setAlwaysOnTop = debounce((value: boolean) => {
    if (value) {
      invoke(SeelenCommand.SetSelfZOrder, { zOrder: ZOrder.TopMost });
    } else {
      invoke(SeelenCommand.SetSelfZOrder, { zOrder: ZOrder.Bottom });
    }
  }, 200);

  $effect(() => {
    setAlwaysOnTop(alwaysOnTop);
  });

  onMount(() => {
    Widget.self.ready().then(() => {
      settingsState.isReady = true;
    });
  });
</script>

<Dock />
