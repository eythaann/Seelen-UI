export * from "./LazyRune.svelte";
export * from "./PersistentRune.svelte";
export * from "./i18n";

/** Get relative time string from a date (e.g., "2 hours ago", "3 days ago") */
export function relativeTimeFromNow(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);
  const diffWeeks = Math.floor(diffDays / 7);
  const diffMonths = Math.floor(diffDays / 30);
  const diffYears = Math.floor(diffDays / 365);

  const rtf = new Intl.RelativeTimeFormat(navigator.language, { numeric: "auto" });

  if (diffYears > 0) {
    return rtf.format(-diffYears, "year");
  } else if (diffMonths > 0) {
    return rtf.format(-diffMonths, "month");
  } else if (diffWeeks > 0) {
    return rtf.format(-diffWeeks, "week");
  } else if (diffDays > 0) {
    return rtf.format(-diffDays, "day");
  } else if (diffHours > 0) {
    return rtf.format(-diffHours, "hour");
  } else if (diffMinutes > 0) {
    return rtf.format(-diffMinutes, "minute");
  } else {
    return rtf.format(-diffSeconds, "second");
  }
}
