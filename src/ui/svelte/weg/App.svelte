<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { ZOrder } from "@seelen-ui/lib/types";
  import { onMount } from "svelte";
  import Dock from "./components/Dock.svelte";
  import { settingsState } from "./state/settings.svelte.ts";

  onMount(() => {
    Widget.self.ready().then(() => {
      settingsState.isReady = true;

      // Keep WEG/Dock above normal application windows.
      // Auto-hide behavior remains controlled separately.
      invoke(SeelenCommand.SetSelfZOrder, {
        zOrder: ZOrder.TopMost,
      });
    });
  });
</script>

<Dock />
