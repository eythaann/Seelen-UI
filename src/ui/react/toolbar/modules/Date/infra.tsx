import { useComputed } from "@preact/signals";
import type { DateToolbarItem } from "@seelen-ui/lib/types";
import { useSyncClockInterval } from "libs/ui/react/utils/hooks.ts";
import moment from "moment";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Item } from "../item/infra/infra.tsx";

import { $settings } from "../shared/state/mod.ts";
import { WithDateCalendar } from "./Calendar.tsx";

interface Props {
  module: DateToolbarItem;
}

const momentJsLangMap: { [key: string]: string } = {
  "no": "nb",
  "zh": "zh-cn",
};

export function DateModule({ module }: Props) {
  const $date_format = useComputed(() => $settings.value.dateFormat);

  const {
    i18n: { language: lang },
  } = useTranslation();
  let language = momentJsLangMap[lang] || lang;

  const [date, setDate] = useState(
    moment().locale(language).format($date_format.value),
  );

  // inmediately update the date, like interval is reseted on deps change
  useEffect(() => {
    setDate(moment().locale(language).format($date_format.value));
  }, [$date_format.value, language]);

  useSyncClockInterval(
    () => {
      setDate(moment().locale(language).format($date_format.value));
    },
    $date_format.value.includes("ss") ? "seconds" : "minutes",
    [$date_format.value, language],
  );

  return (
    <WithDateCalendar>
      <Item extraVars={{ date }} module={module} />
    </WithDateCalendar>
  );
}
