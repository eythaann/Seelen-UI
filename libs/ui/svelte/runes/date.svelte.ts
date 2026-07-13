import moment from "moment";

const momentJsLangMap: Record<string, string> = { no: "nb" };

let _date = $state(Date.now());

let _dateLang = $state("en");
let _dateFormat = $state("ddd D MMM, hh:mm A");

const updatesEverySecond = $derived(_dateFormat.includes("ss"));

$effect.root(() => {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  function tick() {
    _date = Date.now();
    timeout = setTimeout(tick, updatesEverySecond ? 1000 : 60000);
  }

  $effect(() => {
    const now = Date.now();
    const msToSync = updatesEverySecond ? 1000 - (now % 1000) : 60000 - (now % 60000);
    timeout = setTimeout(tick, msToSync);

    return () => {
      if (timeout) clearTimeout(timeout);
    };
  });
});

const _formatedDate = $derived.by(() => {
  const lang = momentJsLangMap[_dateLang] || _dateLang.toLowerCase();

  return moment(_date).locale(lang).format(_dateFormat);
});

class DateState {
  get date() {
    return _date;
  }

  get formatedDate() {
    return _formatedDate;
  }

  setFormat(format: string) {
    _dateFormat = format;
  }

  setLang(lang: string) {
    _dateLang = lang;
  }
}

export const dateState = new DateState();
