const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");
export const prefersDarkColorScheme = $state({ value: darkModeQuery.matches });

darkModeQuery.addEventListener("change", (e) => {
  prefersDarkColorScheme.value = e.matches;
});
