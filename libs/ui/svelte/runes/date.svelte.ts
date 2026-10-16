import moment from "moment";

const momentJsLangMap: Record<string, string> = { no: "nb" };

let _date = $state(Date.now());

let _dateLang = $state("en");
let _dateFormat = $state("ddd D MMM, hh:mm A");

const updatesEverySecond = $derived(_dateFormat.includes("ss"));

$effect.root(() => {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  // The delay is recalculated on every tick. A fixed chained delay (`setTimeout(tick, 60000)`)
  // keeps the lateness of each timer, so the clock slowly drifts away from the minute boundary and
  // never recovers. Each monitor has its own webview, so each one drifts by a different amount.
  function scheduleNextTick() {
    const period = updatesEverySecond ? 1000 : 60000;
    timeout = setTimeout(tick, period - (Date.now() % period));
  }

  function tick() {
    _date = Date.now();
    scheduleNextTick();
  }

  $effect(() => {
    scheduleNextTick();

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
