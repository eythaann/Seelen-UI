import { Settings } from "@seelen-ui/lib";
import type { Settings as SettingsType, StartOfWeek } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import moment from "moment";

let settings = $state<SettingsType>(await Settings.getAsync());
Settings.onChange((s) => (settings = s));

// Local reactive state
let viewMode = $state<"month" | "year">("month");

const momentJsLangMap: { [key: string]: string } = {
  no: "nb",
};

function toMomentLang(lang: string): string {
  return momentJsLangMap[lang] || lang.toLowerCase();
}

const startDayMap: Record<StartOfWeek, number> = {
  Sunday: 0,
  Monday: 1,
  Saturday: 6,
};

$effect.root(() => {
  $effect(() => {
    const lang = settings.language;
    locale.set(lang);

    const momentLang = toMomentLang(lang);
    const startDay = startDayMap[settings.startOfWeek] ?? 0;

    moment.updateLocale(momentLang, {
      week: { dow: startDay },
      postformat: (str: string) => str,
    });
  });
});

class State {
  get viewMode() {
    return viewMode;
  }
  set viewMode(value: "month" | "year") {
    viewMode = value;
  }
  get settings() {
    return settings;
  }
  get momentLang() {
    return toMomentLang(settings.language);
  }
}

export const globalState = new State();
