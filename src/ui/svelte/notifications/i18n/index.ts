import type { SupportedLanguagesCode } from "@seelen-ui/lib";
import { locale, setMessages, t } from "libs/ui/svelte/utils";
import yaml from "js-yaml";

export async function loadTranslations() {
  const translations: Record<SupportedLanguagesCode, { default: string }> = {
    en: await import("./translations/en.yml"),
    es: await import("./translations/en.yml"),
    de: await import("./translations/en.yml"),
    "zh-CN": await import("./translations/en.yml"),
    "zh-TW": await import("./translations/en.yml"),
    ko: await import("./translations/en.yml"),
    fr: await import("./translations/en.yml"),
    ar: await import("./translations/en.yml"),
    ru: await import("./translations/en.yml"),
    "pt-BR": await import("./translations/en.yml"),
    "pt-PT": await import("./translations/en.yml"),
    ja: await import("./translations/en.yml"),
    hi: await import("./translations/en.yml"),
    it: await import("./translations/en.yml"),
    nl: await import("./translations/en.yml"),
    tr: await import("./translations/en.yml"),
    pl: await import("./translations/en.yml"),
    uk: await import("./translations/en.yml"),
    id: await import("./translations/en.yml"),
    cs: await import("./translations/en.yml"),
    th: await import("./translations/en.yml"),
    vi: await import("./translations/en.yml"),
    ms: await import("./translations/en.yml"),
    he: await import("./translations/en.yml"),
    ro: await import("./translations/en.yml"),
    el: await import("./translations/en.yml"),
    sv: await import("./translations/en.yml"),
    no: await import("./translations/en.yml"),
    fi: await import("./translations/en.yml"),
    da: await import("./translations/en.yml"),
    hu: await import("./translations/en.yml"),
    lt: await import("./translations/en.yml"),
    bg: await import("./translations/en.yml"),
    sk: await import("./translations/en.yml"),
    hr: await import("./translations/en.yml"),
    lv: await import("./translations/en.yml"),
    et: await import("./translations/en.yml"),
    tl: await import("./translations/en.yml"),
    ca: await import("./translations/en.yml"),
    af: await import("./translations/en.yml"),
    bn: await import("./translations/en.yml"),
    fa: await import("./translations/en.yml"),
    pa: await import("./translations/en.yml"),
    sw: await import("./translations/en.yml"),
    ta: await import("./translations/en.yml"),
    ur: await import("./translations/en.yml"),
    cy: await import("./translations/en.yml"),
    am: await import("./translations/en.yml"),
    hy: await import("./translations/en.yml"),
    az: await import("./translations/en.yml"),
    eu: await import("./translations/en.yml"),
    bs: await import("./translations/en.yml"),
    ka: await import("./translations/en.yml"),
    gu: await import("./translations/en.yml"),
    is: await import("./translations/en.yml"),
    km: await import("./translations/en.yml"),
    ku: await import("./translations/en.yml"),
    lo: await import("./translations/en.yml"),
    lb: await import("./translations/en.yml"),
    mk: await import("./translations/en.yml"),
    mt: await import("./translations/en.yml"),
    mn: await import("./translations/en.yml"),
    ne: await import("./translations/en.yml"),
    ps: await import("./translations/en.yml"),
    sr: await import("./translations/en.yml"),
    si: await import("./translations/en.yml"),
    so: await import("./translations/en.yml"),
    tg: await import("./translations/en.yml"),
    te: await import("./translations/en.yml"),
    uz: await import("./translations/en.yml"),
    yo: await import("./translations/en.yml"),
    zu: await import("./translations/en.yml"),
  };

  let temp: Record<string, any> = {};
  for (const [key, value] of Object.entries(translations)) {
    temp[key] = yaml.load(value.default);
  }

  setMessages(temp);
}

export { locale, t };
