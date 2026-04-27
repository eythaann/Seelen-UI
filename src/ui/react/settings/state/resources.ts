import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { ResourceText } from "@seelen-ui/lib/types";
import { computed } from "@preact/signals";
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

// readonly configs
export const bundledAppConfigs = lazySignal(() => invoke(SeelenCommand.StateGetSettingsByApp));
subscribe(SeelenEvent.StateSettingsByAppChanged, bundledAppConfigs.setByPayload);
await bundledAppConfigs.init();
