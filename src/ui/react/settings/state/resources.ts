import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { Resource, ResourceKind, ResourceMetadata, ResourceText } from "@seelen-ui/lib/types";
import { ResourceKind as RK } from "@seelen-ui/lib/types";
import { computed, effect, signal } from "@preact/signals";
import { getResourceText } from "@shared";
import { language } from "./mod";

function SorterByDisplayName(
  a: { metadata: { displayName: ResourceText } },
  b: { metadata: { displayName: ResourceText } },
): number {
  const aDisplayName = getResourceText(a.metadata.displayName, language.value);
  const bDisplayName = getResourceText(b.metadata.displayName, language.value);

  return aDisplayName.localeCompare(bDisplayName, language.value);
}

const _widgets = lazySignal(() => invoke(SeelenCommand.StateGetWidgets));
subscribe(SeelenEvent.StateWidgetsChanged, _widgets.setByPayload);
await _widgets.init();

const _plugins = lazySignal(() => invoke(SeelenCommand.StateGetPlugins));
subscribe(SeelenEvent.StatePluginsChanged, _plugins.setByPayload);
await _plugins.init();

const _themes = lazySignal(() => invoke(SeelenCommand.StateGetThemes));
subscribe(SeelenEvent.StateThemesChanged, _themes.setByPayload);
await _themes.init();

const _iconPacks = lazySignal(() => invoke(SeelenCommand.StateGetIconPacks));
subscribe(SeelenEvent.StateIconPacksChanged, _iconPacks.setByPayload);
await _iconPacks.init();

const _wallpapers = lazySignal(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, _wallpapers.setByPayload);
await _wallpapers.init();

export const widgets = computed(() => _widgets.value.toSorted(SorterByDisplayName));
export const plugins = computed(() => _plugins.value.toSorted(SorterByDisplayName));
export const themes = computed(() => _themes.value.toSorted(SorterByDisplayName));
export const iconPacks = computed(() => _iconPacks.value.toSorted(SorterByDisplayName));
export const wallpapers = computed(() => _wallpapers.value.sort(SorterByDisplayName));

// check for updates
export type ResourceWithKind = { id: string; metadata: ResourceMetadata; kind: ResourceKind };
export const resourcesWithUpdate = signal<ResourceWithKind[]>([]);

let _checkController: AbortController | null = null;

effect(() => {
  const allResources: ResourceWithKind[] = [
    ...widgets.value.map((r) => ({ id: r.id, metadata: r.metadata, kind: RK.Widget })),
    ...themes.value.map((r) => ({ id: r.id, metadata: r.metadata, kind: RK.Theme })),
    ...plugins.value.map((r) => ({ id: r.id, metadata: r.metadata, kind: RK.Plugin })),
    ...iconPacks.value.map((r) => ({ id: r.id, metadata: r.metadata, kind: RK.IconPack })),
    ...wallpapers.value.map((r) => ({ id: r.id, metadata: r.metadata, kind: RK.Wallpaper })),
  ].filter((r) => !r.id.startsWith("@"));

  _checkController?.abort();
  _checkController = new AbortController();
  const abortSignal = _checkController.signal;

  Promise.allSettled(
    allResources.map(async (resource) => {
      const res = await fetch(`https://product.seelen.io/resource/${resource.id}`, {
        signal: abortSignal,
      });
      if (!res.ok) return null;
      const remote: Resource = await res.json();
      if (new Date(remote.updatedAt) > new Date(resource.metadata.writtenAt)) {
        return resource;
      }
      return null;
    }),
  ).then((results) => {
    if (abortSignal.aborted) return;
    resourcesWithUpdate.value = results
      .filter(
        (r): r is PromiseFulfilledResult<ResourceWithKind> => r.status === "fulfilled" && r.value !== null,
      )
      .map((r) => r.value);
  });
});
