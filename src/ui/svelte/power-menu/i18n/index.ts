import type { SupportedLanguagesCode } from "@seelen-ui/lib";
import yaml from "js-yaml";
import { derived, get, writable } from "svelte/store";

export const locale = writable("en");
const messages = writable<Record<string, any>>({});

function translate(locale: string, key: string, vars: Record<string, string> = {}) {
  // Let's throw some errors if we're trying to use keys/locales that don't exist.
  // We could improve this by using Typescript and/or fallback values.
  if (!key) throw new Error("no key provided to $t()");
  if (!locale) throw new Error(`no translation for key "${key}"`);

  // Grab the translation from the translations object.
  let text = get(messages)[locale]?.[key];
  if (!text) {
    console.error(`no translation found for ${locale}.${key}`);
    text = get(messages)["en"]?.[key] || key;
  }

  if (typeof text !== "string") {
    console.error(`translation for ${locale}.${key} is not a string`);
    text = key;
  }

  // Replace any passed in variables in the translation string.
  Object.entries(vars).map(([k, v]) => {
    text = text.replaceAll(`{{${k}}}`, v);
  });

  return text;
}

export const t = derived(
  locale,
  ($locale) => (key: string, vars?: Record<string, string>) => translate($locale, key, vars),
);

export async function loadTranslations() {
  const translations: Record<SupportedLanguagesCode, { default: string }> = {
    en: await import("./translations/en.yml"),
    es: await import("./translations/es.yml"),
    de: await import("./translations/de.yml"),
    "zh-CN": await import("./translations/zh-CN.yml"),
    "zh-TW": await import("./translations/zh-TW.yml"),
    ko: await import("./translations/ko.yml"),
    fr: await import("./translations/fr.yml"),
    ar: await import("./translations/ar.yml"),
    ru: await import("./translations/ru.yml"),
    "pt-BR": await import("./translations/pt-BR.yml"),
    "pt-PT": await import("./translations/pt-PT.yml"),
    ja: await import("./translations/ja.yml"),
    hi: await import("./translations/hi.yml"),
    it: await import("./translations/it.yml"),
    nl: await import("./translations/nl.yml"),
    tr: await import("./translations/tr.yml"),
    pl: await import("./translations/pl.yml"),
    uk: await import("./translations/uk.yml"),
    id: await import("./translations/id.yml"),
    cs: await import("./translations/cs.yml"),
    th: await import("./translations/th.yml"),
    vi: await import("./translations/vi.yml"),
    ms: await import("./translations/ms.yml"),
    he: await import("./translations/he.yml"),
    ro: await import("./translations/ro.yml"),
    el: await import("./translations/el.yml"),
    sv: await import("./translations/sv.yml"),
    no: await import("./translations/no.yml"),
    fi: await import("./translations/fi.yml"),
    da: await import("./translations/da.yml"),
    hu: await import("./translations/hu.yml"),
    lt: await import("./translations/lt.yml"),
    bg: await import("./translations/bg.yml"),
    sk: await import("./translations/sk.yml"),
    hr: await import("./translations/hr.yml"),
    lv: await import("./translations/lv.yml"),
    et: await import("./translations/et.yml"),
    tl: await import("./translations/tl.yml"),
    ca: await import("./translations/ca.yml"),
    af: await import("./translations/af.yml"),
    bn: await import("./translations/bn.yml"),
    fa: await import("./translations/fa.yml"),
    pa: await import("./translations/pa.yml"),
    sw: await import("./translations/sw.yml"),
    ta: await import("./translations/ta.yml"),
    ur: await import("./translations/ur.yml"),
    cy: await import("./translations/cy.yml"),
    am: await import("./translations/am.yml"),
    hy: await import("./translations/hy.yml"),
    az: await import("./translations/az.yml"),
    eu: await import("./translations/eu.yml"),
    bs: await import("./translations/bs.yml"),
    ka: await import("./translations/ka.yml"),
    gu: await import("./translations/gu.yml"),
    is: await import("./translations/is.yml"),
    km: await import("./translations/km.yml"),
    ku: await import("./translations/ku.yml"),
    lo: await import("./translations/lo.yml"),
    lb: await import("./translations/lb.yml"),
    mk: await import("./translations/mk.yml"),
    mt: await import("./translations/mt.yml"),
    mn: await import("./translations/mn.yml"),
    ne: await import("./translations/ne.yml"),
    ps: await import("./translations/ps.yml"),
    sr: await import("./translations/sr.yml"),
    si: await import("./translations/si.yml"),
    so: await import("./translations/so.yml"),
    tg: await import("./translations/tg.yml"),
    te: await import("./translations/te.yml"),
    uz: await import("./translations/uz.yml"),
    yo: await import("./translations/yo.yml"),
    zu: await import("./translations/zu.yml"),
  };

  let temp: Record<string, any> = {};
  for (const [key, value] of Object.entries(translations)) {
    temp[key] = yaml.load(value.default);
  }

  messages.set(temp);
}
