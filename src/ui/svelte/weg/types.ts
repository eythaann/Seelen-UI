import type { WegItem } from "@seelen-ui/lib/types";

export type AppOrFileWegItem = Extract<WegItem, { type: "AppOrFile" }>;
export type SeparatorWegItem = Extract<WegItem, { type: "Separator" }>;
export type MediaWegItem = Extract<WegItem, { type: "Media" }>;
export type PluginWegItem = Extract<WegItem, { type: "Plugin" }>;

/** @alias */
export type SwItem = WegItem;
