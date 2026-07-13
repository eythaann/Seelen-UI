<script lang="ts">
  import { WegItemType, type WegPluginItem as WegPluginPayload } from "@seelen-ui/lib/types";
  import type { SwItem } from "../types.ts";
  import { plugins } from "../state/getters.svelte.ts";
  import UserApplication from "./items/UserApplication.svelte";
  import StartMenu from "./items/StartMenu.svelte";
  import ShowDesktop from "./items/ShowDesktop.svelte";
  import MediaSession from "./items/MediaSession.svelte";
  import Separator from "./items/Separator.svelte";
  import RecycleBin from "./items/RecycleBin.svelte";
  import PluginItem from "./items/PluginItem.svelte";

  interface Props {
    item: SwItem;
    isOverlay?: boolean;
  }

  let { item, isOverlay = false }: Props = $props();

  const pluginPayload = $derived(
    item.type === WegItemType.Plugin
      ? (plugins.value.find((p) => p.id === item.plugin)?.plugin as WegPluginPayload | undefined)
      : undefined,
  );
</script>

{#if item.type === WegItemType.AppOrFile}
  <UserApplication {item} {isOverlay} />
{:else if item.type === WegItemType.StartMenu}
  <StartMenu {item} />
{:else if item.type === WegItemType.ShowDesktop}
  <ShowDesktop {item} />
{:else if item.type === WegItemType.Media}
  <MediaSession {item} />
{:else if item.type === WegItemType.Separator}
  <Separator {item} />
{:else if item.type === WegItemType.TrashBin}
  <RecycleBin {item} />
{:else if item.type === WegItemType.Plugin}
  {#if pluginPayload}
    <PluginItem {item} payload={pluginPayload} />
  {/if}
{/if}
