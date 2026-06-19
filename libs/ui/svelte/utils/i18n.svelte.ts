import yaml from "js-yaml";

let _locale = $state("en");
let _messages = $state.raw<Record<string, any>>({});

function translate(locale: string, key: string, vars: Record<string, string> = {}) {
  // Let's throw some errors if we're trying to use keys/locales that don't exist.
  // We could improve this by using Typescript and/or fallback values.
  if (!key) throw new Error("no key provided to $t()");
  if (!locale) throw new Error(`no translation for key "${key}"`);

  // Grab the translation from the translations object.
  // Support nested keys like "profile.log_out"
  const keys = key.split(".");
  let text = _messages[locale];

  for (const k of keys) {
    text = text?.[k];
  }

  if (!text) {
    console.error(`no translation found for ${locale}.${key}`);
    // Try fallback to English
    let fallback = _messages["en"];
    for (const k of keys) {
      fallback = fallback?.[k];
    }
    text = fallback || key;
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

export function t(key: string, vars?: Record<string, string>) {
  return translate(_locale, key, vars);
}

class i18n {
  get locale() {
    return _locale;
  }

  async loadLocale(newLocale: string) {
    if (_messages[newLocale]) {
      _locale = newLocale;
      return;
    }

    const res = await fetch(`./translations/${newLocale}.yml`);
    const text = await res.text();
    const obj = yaml.load(text) as Record<string, any>;
    _messages = {
      ..._messages,
      [newLocale]: obj,
    };
    _locale = newLocale;
  }
}

export default i18n;
