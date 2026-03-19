import { IconPackManager } from "@seelen-ui/lib";
import { signal } from "@preact/signals";

const manager = await IconPackManager.create();
export const iconPackManager = signal({ _version: 0, value: manager });
manager.onChange(() => {
  iconPackManager.value = { _version: iconPackManager.value._version + 1, value: manager };
});

const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");
export const darkMode = signal(darkModeQuery.matches);
darkModeQuery.addEventListener("change", () => {
  darkMode.value = darkModeQuery.matches;
});
