const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cx = w / 2;

ctx.clearRect(0, 0, w, h);
ctx.lineCap = "round";
ctx.lineJoin = "round";

const color = themeTokens.foregroundColor;
const lineWidth = w * 0.07;

const knobRadius = lineWidth * 0.5;
const clapperRadius = lineWidth * 0.55;
const strokeMargin = lineWidth * 0.3;

const domeRadius = w * 0.3;
const knobCenterY = strokeMargin + knobRadius;
const domeCenterY = knobCenterY + lineWidth * 0.4 + domeRadius;
const clapperCenterY = h - strokeMargin - clapperRadius;
const bottomY = clapperCenterY - lineWidth;
const straightEndY = domeCenterY + (bottomY - domeCenterY) * 0.6;
const bodyLeft = cx - domeRadius * 1.3;
const bodyRight = cx + domeRadius * 1.3;

ctx.strokeStyle = color;
ctx.lineWidth = lineWidth;

// bell body: rounded dome + straight sides + small skirt flare + bottom rim
ctx.beginPath();
ctx.arc(cx, domeCenterY, domeRadius, Math.PI, 0);
ctx.lineTo(cx + domeRadius, straightEndY);
ctx.lineTo(bodyRight, bottomY);
ctx.lineTo(bodyLeft, bottomY);
ctx.lineTo(cx - domeRadius, straightEndY);
ctx.closePath();
ctx.stroke();

// knob on top
ctx.beginPath();
ctx.arc(cx, knobCenterY, knobRadius, 0, Math.PI * 2);
ctx.fillStyle = color;
ctx.fill();

// clapper
ctx.beginPath();
ctx.arc(cx, clapperCenterY, clapperRadius, 0, Math.PI * 2);
ctx.fillStyle = color;
ctx.fill();

if (dndActive) {
  ctx.fillStyle = color;
  ctx.font = `bold ${Math.round(h * 0.3)}px sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("zZ", cx, h * 0.55);
}
