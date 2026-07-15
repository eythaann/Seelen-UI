const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cx = w / 2;
const cy = h / 2;

ctx.clearRect(0, 0, w, h);
ctx.lineCap = "round";
ctx.lineJoin = "round";

const connectedDevices = (devices || []).filter((d) => d.connected);
const hasConnected = connectedDevices.length > 0;

const color = themeTokens.foregroundColor;
const lineWidth = 18;

const top = h * 0.1;
const bottom = h * 0.9;
const quarter = (bottom - top) / 4;
const leftX = cx - w * 0.25;
const rightX = cx + w * 0.25;
const upperY = top + quarter;
const lowerY = bottom - quarter;

ctx.strokeStyle = color;
ctx.lineWidth = lineWidth;

// Bluetooth rune: two crossing diagonals joined by a straight vertical stem
ctx.beginPath();
ctx.moveTo(leftX, upperY);
ctx.lineTo(rightX, lowerY);
ctx.lineTo(cx, bottom);
ctx.lineTo(cx, top);
ctx.lineTo(rightX, upperY);
ctx.lineTo(leftX, lowerY);
ctx.stroke();

if (hasConnected) {
  const dotRadius = lineWidth * 0.55;
  const dotOffset = w * 0.40;

  ctx.fillStyle = color;

  ctx.beginPath();
  ctx.arc(cx - dotOffset, cy, dotRadius, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(cx + dotOffset, cy, dotRadius, 0, Math.PI * 2);
  ctx.fill();
}
