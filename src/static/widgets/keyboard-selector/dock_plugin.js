const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cx = w / 2;

ctx.clearRect(0, 0, w, h);
ctx.textAlign = "center";
ctx.textBaseline = "middle";

const lang = activeLangPrefix || "Null";
const variant = activeKeyboardPrefix || "";
const inputsLength = activeLang.keyboardLayouts.length;

if (variant && inputsLength > 1) {
  ctx.fillStyle = themeTokens.foregroundColor;
  ctx.font = `700 ${h * 0.4}px sans-serif`;
  ctx.fillText(lang, cx, h * 0.42);

  ctx.fillStyle = themeTokens.foregroundSecondaryColor;
  ctx.font = `600 ${h * 0.35}px sans-serif`;
  ctx.fillText(variant, cx, h * 0.75);
} else {
  ctx.fillStyle = themeTokens.foregroundColor;
  ctx.font = `700 ${h * 0.4}px sans-serif`;
  ctx.fillText(lang, cx, h / 2);
}
