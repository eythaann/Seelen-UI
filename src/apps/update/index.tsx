import { UserSettingsLoader } from '../settings/modules/shared/store/storeApi';
import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import i18n, { loadTranslations } from './i18n';
import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';

import { App } from './app';

import './styles/colors.css';
import './styles/reset.css';
import './styles/global.css';

async function main() {
  const container = getRootContainer();
  wrapConsole();

  let { jsonSettings } = await new UserSettingsLoader().withThemes(false).load();
  await loadTranslations();
  i18n.changeLanguage(jsonSettings.language);

  createRoot(container).render(
    <I18nextProvider i18n={i18n}>
      <App />
    </I18nextProvider>,
  );
}

main();