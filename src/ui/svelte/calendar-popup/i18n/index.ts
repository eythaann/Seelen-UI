import type { SupportedLanguagesCode } from "@seelen-ui/lib";
import { locale, setMessages, t } from "libs/ui/svelte/utils";
import yaml from "js-yaml";

export async function loadTranslations() {
  const translations: Record<SupportedLanguagesCode, { default: string }> = {
    en: await import("./translations/en.yml"),
  } as any;

  let temp: Record<string, any> = {};
  for (const [key, value] of Object.entries(translations)) {
    temp[key] = yaml.load(value.default);
  }

  setMessages(temp);
}

export { locale, t };
