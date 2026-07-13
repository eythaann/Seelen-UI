import { derived, get, writable } from "svelte/store";
import yaml from "js-yaml";

const _locale = writable("en");
const _messages = writable<Record<string, any>>({});

function translate(locale: string, key: string, vars: Record<string, string> = {}) {
  // Let's throw some errors if we're trying to use keys/locales that don't exist.
  // We could improve this by using Typescript and/or fallback values.
  if (!key) throw new Error("no key provided to $t()");
  if (!locale) throw new Error(`no translation for key "${key}"`);

  // Grab the translation from the translations object.
  // Support nested keys like "profile.log_out"
  const keys = key.split(".");
  let text = get(_messages)[locale];
  for (const k of keys) {
    text = text?.[k];
  }

  if (!text) {
    console.error(`no translation found for ${locale}.${key}`);
    // Try fallback to English
    let fallback = get(_messages)["en"];
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

export const t = derived(
  _locale,
  (locale) => (key: string, vars?: Record<string, string>) => translate(locale, key, vars),
);

async function loadLocale(locale: string) {
  // If the locale is already loaded, don't load it again
  if (get(_messages)[locale]) {
    return;
  }
  const res = await fetch(`./translations/${locale}.yml`);
  const text = await res.text();
  const messages = yaml.load(text) as Record<string, any>;
  _messages.update((m) => ({ ...m, [locale]: messages }));
}

// Load the default locale in the background
loadLocale("en");

export const locale = {
  get value() {
    return get(_locale);
  },

  async set(newLocale: string) {
    await loadLocale(newLocale);
    _locale.set(newLocale);
  },
};

export function setMessages(newMessages: Record<string, any>) {
  _messages.set(newMessages);
}
