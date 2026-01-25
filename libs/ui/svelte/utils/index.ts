export * from "./LazyRune.svelte";
export * from "./PersistentRune.svelte";
export * from "./i18n";

// Difference between Windows epoch (1601) and Unix epoch (1970) in milliseconds
const EPOCH_DIFF_MILLISECONDS = 11644473600000n;

/** Convert Windows FileTime to Js Unix Date */
export function WindowsDateFileTimeToDate(fileTime: bigint | number): Date {
  if (typeof fileTime === "number") fileTime = BigInt(fileTime);
  return new Date(Number(fileTime / 10000n - EPOCH_DIFF_MILLISECONDS));
}

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
