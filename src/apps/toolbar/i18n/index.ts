import { Lang } from '../../shared/lang';
import i18n from 'i18next';
import yaml from 'js-yaml';
import { initReactI18next } from 'react-i18next';

i18n.use(initReactI18next).init(
  {
    lng: 'en',
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
    debug: true,
    resources: {},
  },
  undefined,
);

export async function loadTranslations() {
  const translations: Record<Lang, { default: string }> = {
    en: await import('./translations/en.yml'),
    es: await import('./translations/es.yml'),
    de: await import('./translations/de.yml'),
    zh: await import('./translations/zh.yml'),
    ko: await import('./translations/ko.yml'),
    fr: await import('./translations/fr.yml'),
    ar: await import('./translations/ar.yml'),
    pt: await import('./translations/pt.yml'),
  };

  for (const [key, value] of Object.entries(translations)) {
    i18n.addResourceBundle(key, 'translation', yaml.load(value.default));
  }
}

export default i18n;
