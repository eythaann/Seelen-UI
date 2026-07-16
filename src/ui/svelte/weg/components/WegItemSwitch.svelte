<script lang="ts">
  import type { WegPluginItem as WegPluginPayload } from "@seelen-ui/lib/types";
  import type { SwItem } from "../types.ts";
  import { plugins } from "../state/getters.svelte.ts";
  import UserApplication from "./items/UserApplication.svelte";
  import MediaSession from "./items/MediaSession.svelte";
  import Separator from "./items/Separator.svelte";
  import PluginItem from "./items/PluginItem.svelte";

  interface Props {
    item: SwItem;
    isOverlay?: boolean;
  }

  let { item, isOverlay = false }: Props = $props();

  const pluginPayload = $derived(
    item.type === "Plugin"
      ? (plugins.value.find((p) => p.id === item.plugin)?.plugin as WegPluginPayload | undefined)
      : undefined,
  );
</script>

{#if item.type === "AppOrFile"}
  <UserApplication {item} {isOverlay} />
{:else if item.type === "Media"}
  <MediaSession {item} />
{:else if item.type === "Separator"}
  <Separator {item} />
{:else if item.type === "Plugin"}
  {#if pluginPayload}
    <PluginItem {item} payload={pluginPayload} />
  {/if}
{/if}
