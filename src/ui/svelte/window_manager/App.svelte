<script lang="ts">
  import { onMount } from "svelte";
  import { Widget } from "@seelen-ui/lib";
  import Layout from "./modules/layout/infra/Layout.svelte";

  onMount(() => {
    let widget = Widget.getCurrent();
    widget.webview.emitTo(widget.webview.label, "complete-setup");
    widget.ready();
  });
</script>

<Layout />
