<script lang="ts">
  import { onMount } from "svelte";
  import { Widget } from "@seelen-ui/lib";
  import Layout from "./layout/infra/Layout.svelte";

  onMount(() => {
    Widget.getCurrent().ready();
  });
</script>

<Layout monitorId={Widget.getCurrent().decoded.monitorId!} />
