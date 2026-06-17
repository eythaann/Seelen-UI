import type { WegItem } from "@seelen-ui/lib/types";

export type AppOrFileWegItem = Extract<WegItem, { type: "AppOrFile" }>;
export type SeparatorWegItem = Extract<WegItem, { type: "Separator" }>;
export type MediaWegItem = Extract<WegItem, { type: "Media" }>;
export type StartMenuWegItem = Extract<WegItem, { type: "StartMenu" }>;
export type ShowDesktopWegItem = Extract<WegItem, { type: "ShowDesktop" }>;
export type TrashBinItem = Extract<WegItem, { type: "TrashBin" }>;

/** @alias */
export type SwItem = WegItem;
