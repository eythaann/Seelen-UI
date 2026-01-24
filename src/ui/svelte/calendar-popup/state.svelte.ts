import { Settings } from "@seelen-ui/lib";
import type { Settings as SettingsType } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import moment from "moment";

let settings = $state<SettingsType>(await Settings.getAsync());
Settings.onChange((s) => (settings = s));

// Local reactive state
let date = $state(moment());
let selectedDate = $state(moment());
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
    moment.locale(language);

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

    // Update dates with new locale
    date = date.locale(language);
    selectedDate = selectedDate.locale(language);
  });
});

class State {
  get date() {
    return date;
  }
  set date(value: moment.Moment) {
    date = value;
  }

  get selectedDate() {
    return selectedDate;
  }
  set selectedDate(value: moment.Moment) {
    selectedDate = value;
  }

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
