import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

export const widgets = lazySignal(() => invoke(SeelenCommand.StateGetWidgets));
subscribe(SeelenEvent.StateWidgetsChanged, widgets.setByPayload);
await widgets.init();

export const plugins = lazySignal(() => invoke(SeelenCommand.StateGetPlugins));
subscribe(SeelenEvent.StatePluginsChanged, plugins.setByPayload);
await plugins.init();

export const themes = lazySignal(() => invoke(SeelenCommand.StateGetThemes));
subscribe(SeelenEvent.StateThemesChanged, themes.setByPayload);
await themes.init();

export const iconPacks = lazySignal(() => invoke(SeelenCommand.StateGetIconPacks));
subscribe(SeelenEvent.StateIconPacksChanged, iconPacks.setByPayload);
await iconPacks.init();

export const wallpapers = lazySignal(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);
await wallpapers.init();

// readonly configs
export const bundledAppConfigs = lazySignal(() => invoke(SeelenCommand.StateGetSettingsByApp));
subscribe(SeelenEvent.StateSettingsByAppChanged, bundledAppConfigs.setByPayload);
await bundledAppConfigs.init();
