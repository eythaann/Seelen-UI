<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { ZOrder } from "@seelen-ui/lib/types";
  import { onMount } from "svelte";
  import { debounce } from "lodash";
  import Toolbar from "./components/Toolbar.svelte";
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

  // Windows can leave the toolbar with WS_EX_TOPMOST set but stacked below normal windows (seen
  // when a WinUI app like Photos opens), and from then on every focused app covers it. Showing
  // the widget doesn't restack it, so the topmost position is re-asserted after it is shown and
  // after every focus change.
  const reassertTopMost = debounce(() => {
    invoke(SeelenCommand.SetSelfZOrder, { zOrder: ZOrder.TopMost });
  }, 250);

  const showWidget = $derived(!topWindowIsFullscreen || focusedIsAppsMenu);
  // only the focused window identity matters; title or position updates of the same window must not
  // restart the debounced repair
  const focusedHwnd = $derived(focused.value?.hwnd);
  const setWidgetVisibility = debounce((value: boolean) => {
    if (value) {
      Widget.self.show();
      reassertTopMost();
    } else {
      Widget.self.hide();
    }
  }, 100);

  $effect(() => {
    if (settingsState.isReady) {
      setWidgetVisibility(showWidget);
    }
  });

  $effect(() => {
    // tracked on purpose: any focus change can push the toolbar out of the topmost band
    focusedHwnd;
    if (settingsState.isReady && showWidget) {
      reassertTopMost();
    }
  });

  onMount(() => {
    Widget.self.ready({ show: false }).then(() => {
      settingsState.isReady = true;
    });
  });
</script>

<Toolbar />
