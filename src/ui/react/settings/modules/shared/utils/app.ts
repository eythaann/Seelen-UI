import type { TFunction } from "i18next";

import type { HexColor } from "./domain.ts";

type Args = undefined | string | { [x: string]: boolean | null | undefined };
export const cx = (...args: Args[]): string => {
  return args
    .map((arg) => {
      if (!arg) {
        return;
      }

      if (typeof arg === "string") {
        return arg;
      }

      return Object.keys(arg)
        .map((key) => (arg[key] ? key : ""))
        .join(" ");
    })
    .join(" ");
};

export const capitalize = (text: string) => {
  return text.slice(0, 1).toUpperCase() + text.slice(1);
};

export const defaultOnNull = <T>(value: T | null | undefined, onNull: T): T => {
  if (value == null) {
    return onNull;
  }
  return value;
};

export const validateHexColor = (str: string): HexColor | null => {
  if (!str.startsWith("#")) {
    return null;
  }
  return str as HexColor;
};

export const OptionsFromEnum = (
  t: TFunction,
  obj: anyObject,
  translationPrefix: string,
) => {
  return Object.values(obj).map((value) => ({
    label: t(translationPrefix + "." + toSnakeCase(value)),
    value,
  }));
};

function toSnakeCase(text: string) {
  let snake = "";
  for (let i = 0; i < text.length; i++) {
    const char = text[i]!;
    if (char === char.toLowerCase() && !char.match(/[0-9]/)) {
      snake += char.toLowerCase();
    } else if (i == 0) {
      snake += `${char.toLowerCase()}`;
    } else {
      snake += `_${char.toLowerCase()}`;
    }
  }
  return snake;
}
