<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { ZOrder } from "@seelen-ui/lib/types";
  import { onMount } from "svelte";
  import { debounce } from "lodash";
  import Dock from "./components/Dock.svelte";
  import { topInteractableWindow, focused, widgetStatuses } from "./state/windows.svelte.ts";

  const startMenuExes = ["SearchHost.exe", "StartMenuExperienceHost.exe"];

  const topWindowIsFullscreen = $derived(topInteractableWindow.value?.isFullscreen);

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
    Widget.self.ready();
  });
</script>

<Dock />
