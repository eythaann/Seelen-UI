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

export const AVAILABLE_LANGUAGES = [
  { label: 'English', value: 'en' },
  { label: 'Español', value: 'es' },
  { label: 'Deutsch', value: 'de' },
  { label: '中文', value: 'zh' },
  { label: '한국어', value: 'ko' },
].sort((a, b) => a.value.localeCompare(b.value));

export async function loadTranslations() {
  const translations = {
    en: await import('./translations/en.yml'),
    es: await import('./translations/es.yml'),
    de: await import('./translations/de.yml'),
    zh: await import('./translations/zh.yml'),
    ko: await import('./translations/ko.yml'),
  };

  for (const [key, value] of Object.entries(translations)) {
    i18n.addResourceBundle(key, 'translation', yaml.load(value.default));
  }
}

export default i18n;
