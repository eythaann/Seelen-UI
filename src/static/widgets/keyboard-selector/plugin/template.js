if (!activeLang || !activeKeyboard) return "!?";

const lang = activeLang.code || "";
const ime = imeState;

function getImeChar() {
  if (!ime) return null;

  // Japanese — ja-JP
  if (lang.startsWith("ja")) {
    if (!ime.open || (!ime.native && !ime.fullShape)) return "A"; // Direct Input / Half-width Alphanumeric
    if (!ime.native && ime.fullShape) return "Ａ"; // Full-width Alphanumeric
    if (ime.native && !ime.katakana) return "あ"; // Hiragana
    if (ime.native && ime.katakana && ime.fullShape) return "ア"; // Full-width Katakana
    if (ime.native && ime.katakana && !ime.fullShape) return "ｱ"; // Half-width Katakana
  }

  // Chinese — zh-CN, zh-TW, zh-HK, etc.
  if (lang.startsWith("zh")) {
    if (!ime.open || !ime.native) return "英"; // English / direct input
    return "中"; // Chinese input
  }

  // Korean — ko-KR
  if (lang.startsWith("ko")) {
    if (!ime.open || !ime.native) return "A"; // English
    if (ime.hanja) return "漢"; // Hanja conversion
    return "한"; // Hangul
  }

  return null;
}

const imeChar = getImeChar();

const inputsLength = activeLang.keyboardLayouts.length;
return [
  imeChar,
  imeChar ? " | " : "",
  activeLangPrefix,
  inputsLength > 1 ? " - " : "",
  inputsLength > 1 ? activeKeyboardPrefix : "",
];
