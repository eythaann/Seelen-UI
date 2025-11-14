<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import Layout from "./modules/layout/infra/Layout.svelte";

  onMount(() => {
    let view = getCurrentWebviewWindow();
    view.show();
    view.emitTo(view.label, "complete-setup");
  });
</script>

<Layout />
