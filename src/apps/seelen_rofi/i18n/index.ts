import i18n from 'i18next';
import yaml from 'js-yaml';
import { initReactI18next } from 'react-i18next';

import { SupportedLanguagesCode } from '../../shared/lang';

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
  const translations: Record<SupportedLanguagesCode, { default: string }> = {
    en: await import('./translations/en.yml'),
    es: await import('./translations/es.yml'),
    de: await import('./translations/de.yml'),
    zh: await import('./translations/zh.yml'),
    ko: await import('./translations/ko.yml'),
    fr: await import('./translations/fr.yml'),
    ar: await import('./translations/ar.yml'),
    ru: await import('./translations/ru.yml'),
    pt: await import('./translations/pt.yml'),
    ja: await import('./translations/ja.yml'),
    hi: await import('./translations/hi.yml'),
    it: await import('./translations/it.yml'),
    nl: await import('./translations/nl.yml'),
    tr: await import('./translations/tr.yml'),
    pl: await import('./translations/pl.yml'),
    uk: await import('./translations/uk.yml'),
    id: await import('./translations/id.yml'),
    cs: await import('./translations/cs.yml'),
    th: await import('./translations/th.yml'),
    vi: await import('./translations/vi.yml'),
    ms: await import('./translations/ms.yml'),
    he: await import('./translations/he.yml'),
    ro: await import('./translations/ro.yml'),
    el: await import('./translations/el.yml'),
    sv: await import('./translations/sv.yml'),
    no: await import('./translations/no.yml'),
    fi: await import('./translations/fi.yml'),
    da: await import('./translations/da.yml'),
    hu: await import('./translations/hu.yml'),
    lt: await import('./translations/lt.yml'),
    bg: await import('./translations/bg.yml'),
    sk: await import('./translations/sk.yml'),
    hr: await import('./translations/hr.yml'),
    lv: await import('./translations/lv.yml'),
    et: await import('./translations/et.yml'),
    tl: await import('./translations/tl.yml'),
    ca: await import('./translations/ca.yml'),
    af: await import('./translations/af.yml'),
    bn: await import('./translations/bn.yml'),
    fa: await import('./translations/fa.yml'),
    pa: await import('./translations/pa.yml'),
    sw: await import('./translations/sw.yml'),
    ta: await import('./translations/ta.yml'),
    ur: await import('./translations/ur.yml'),
    cy: await import('./translations/cy.yml'),
    am: await import('./translations/am.yml'),
    hy: await import('./translations/hy.yml'),
    az: await import('./translations/az.yml'),
    eu: await import('./translations/eu.yml'),
    bs: await import('./translations/bs.yml'),
    ka: await import('./translations/ka.yml'),
    gu: await import('./translations/gu.yml'),
    is: await import('./translations/is.yml'),
    km: await import('./translations/km.yml'),
    ku: await import('./translations/ku.yml'),
    lo: await import('./translations/lo.yml'),
    lb: await import('./translations/lb.yml'),
    mk: await import('./translations/mk.yml'),
    mt: await import('./translations/mt.yml'),
    mn: await import('./translations/mn.yml'),
    ne: await import('./translations/ne.yml'),
    ps: await import('./translations/ps.yml'),
    sr: await import('./translations/sr.yml'),
    si: await import('./translations/si.yml'),
    so: await import('./translations/so.yml'),
    tg: await import('./translations/tg.yml'),
    te: await import('./translations/te.yml'),
    uz: await import('./translations/uz.yml'),
    yo: await import('./translations/yo.yml'),
    zu: await import('./translations/zu.yml'),
  };

  for (const [key, value] of Object.entries(translations)) {
    i18n.addResourceBundle(key, 'translation', yaml.load(value.default));
  }
}

export default i18n;
