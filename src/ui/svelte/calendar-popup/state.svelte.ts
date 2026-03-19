import { Settings } from "@seelen-ui/lib";
import type { Settings as SettingsType } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import moment from "moment";

let settings = $state<SettingsType>(await Settings.getAsync());
Settings.onChange((s) => (settings = s));

// Local reactive state
let viewMode = $state<"month" | "year">("month");

const momentJsLangMap: { [key: string]: string } = {
  no: "nb",
  zh: "zh-cn",
};

$effect.root(() => {
  $effect(() => {
    const lang = settings.language || "en";
    locale.set(lang);

    const language = momentJsLangMap[lang] || lang;

    // Update the start of week based on settings
    const startDayMap: Record<string, number> = {
      Sunday: 0,
      Monday: 1,
      Saturday: 6,
    };
    const startDay = startDayMap[settings.startOfWeek] ?? 0;

    moment.updateLocale(language, {
      week: {
        dow: startDay,
      },
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
}

export const globalState = new State();
