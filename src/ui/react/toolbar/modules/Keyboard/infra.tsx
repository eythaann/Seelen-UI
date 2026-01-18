import type { KeyboardToolbarItem } from "@seelen-ui/lib/types";
import { useSelector } from "react-redux";

import { Item } from "../item/infra/infra.tsx";

import { Selectors } from "../shared/store/app.ts";

interface Props {
  module: KeyboardToolbarItem;
}

export function KeyboardModule({ module }: Props) {
  const languages = useSelector(Selectors.languages);

  const activeLang = languages.find((l) => l.keyboardLayouts.some((k) => k.active)) || languages[0];
  const activeKeyboard = activeLang?.keyboardLayouts.find((k) => k.active) || activeLang?.keyboardLayouts[0];

  if (!activeLang || !activeKeyboard) {
    console.error("No active keyboard for unknown reason");
    return null;
  }

  let activeLangPrefix = activeLang.nativeName
    .split("")
    .slice(0, 3)
    .filter((c) => !["(", ")", " "].includes(c))
    .join("")
    .toLocaleUpperCase();

  let words = activeKeyboard.displayName.split(/[\s\-\(\)]/);
  let activeKeyboardPrefix = words.length > 1
    ? words
      .map((word) => word[0])
      .join("")
      .toLocaleUpperCase()
    : words[0]?.slice(0, 3).toLocaleUpperCase() || "";

  return (
    <Item
      extraVars={{
        activeLang,
        activeKeyboard,
        activeLangPrefix,
        activeKeyboardPrefix,
        languages,
      }}
      module={module}
    />
  );
}
