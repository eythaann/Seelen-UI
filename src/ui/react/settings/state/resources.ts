import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

export const widgets = lazySignal(() => invoke(SeelenCommand.StateGetWidgets));
await subscribe(SeelenEvent.StateWidgetsChanged, widgets.setByPayload);
await widgets.init();

export const plugins = lazySignal(() => invoke(SeelenCommand.StateGetPlugins));
await subscribe(SeelenEvent.StatePluginsChanged, plugins.setByPayload);
await plugins.init();

export const themes = lazySignal(() => invoke(SeelenCommand.StateGetThemes));
await subscribe(SeelenEvent.StateThemesChanged, themes.setByPayload);
await themes.init();

export const iconPacks = lazySignal(() => invoke(SeelenCommand.StateGetIconPacks));
await subscribe(SeelenEvent.StateIconPacksChanged, iconPacks.setByPayload);
await iconPacks.init();

export const wallpapers = lazySignal(() => invoke(SeelenCommand.StateGetWallpapers));
await subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);
await wallpapers.init();

export const appsConfigs = lazySignal(() => invoke(SeelenCommand.StateGetSpecificAppsConfigurations));
await subscribe(SeelenEvent.StateSettingsByAppChanged, appsConfigs.setByPayload);
await appsConfigs.init();
