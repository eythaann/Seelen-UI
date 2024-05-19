import { UserSettings } from '../../../../../shared.interfaces';
import { configureStore } from '@reduxjs/toolkit';

export const store = configureStore({
  reducer: {},
});

export async function loadStore(_userSettings?: UserSettings) {
  /* const userSettings = _userSettings || (await loadUserSettings());
  const settings = userSettings.jsonSettings.fancyToolbar;

  loadSettingsCSS(settings);
  store.dispatch(RootActions.setSettings(settings));

  if (userSettings.theme) {
    loadThemeCSS(userSettings.theme);
    store.dispatch(RootActions.setTheme(userSettings.theme));
  }

  const placeholder =
    userSettings.placeholders.find(
      (placeholder) => placeholder.info.filename === settings.placeholder,
    ) || null;
  store.dispatch(RootActions.setPlaceholder(placeholder));
  store.dispatch(RootActions.setEnv(userSettings.env)); */
}