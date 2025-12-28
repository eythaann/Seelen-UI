import { useEffect, useState } from "react";

type Args = undefined | string | { [x: string]: any };
export const cx = (...args: Args[]): string => {
  return args
    .map((arg) => {
      if (!arg) {
        return;
      }

      if (typeof arg === "string") {
        return arg;
      }

      let classnames = "";
      Object.keys(arg).forEach((key) => {
        if (arg[key]) {
          classnames += ` ${key}`;
        }
      });

      return classnames.trimStart();
    })
    .join(" ");
};

export function isDarkModeEnabled() {
  return globalThis.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function useDarkMode() {
  const [isDarkMode, setIsDarkMode] = useState(isDarkModeEnabled());

  useEffect(() => {
    const mediaQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");
    const listener = () => setIsDarkMode(mediaQuery.matches);
    mediaQuery.addEventListener("change", listener);
    return () => mediaQuery.removeEventListener("change", listener);
  }, []);

  return isDarkMode;
}
