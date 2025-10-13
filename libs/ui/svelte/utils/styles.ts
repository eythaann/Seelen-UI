export function isDarkModeEnabled() {
  return globalThis.matchMedia("(prefers-color-scheme: dark)").matches;
}
