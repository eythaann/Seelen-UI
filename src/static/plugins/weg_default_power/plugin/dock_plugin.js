const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cy = h / 2;

ctx.clearRect(0, 0, w, h);
ctx.lineCap = "round";
ctx.lineJoin = "round";

const hasBattery = batteries.length > 0;
const isCharging = hasBattery &&
  (power?.acLineStatus === 1 || batteries.some((battery) => battery.state === "charging"));
const percentage = hasBattery
  ? batteries.reduce((total, battery) => total + battery.percentage, 0) / batteries.length
  : 0;

const color = themeTokens.foregroundColor;
const accent = systemTokens.accentColor;
const dangerColor = "#e5484d";
const lineWidth = w * 0.06;

function roundedRectPath(x, y, width, height, radius) {
  ctx.beginPath();
  ctx.roundRect(x, y, width, height, radius);
}

// battery is oriented right-to-left (terminal cap on the left) and occupies
// the full canvas width, from x=0 to x=w
const bodyH = h * 0.62;
const capW = w * 0.08;
const capH = bodyH * 0.6;
const capX = 0;
const capY = cy - capH / 2;

const bodyX = capW * 1.1;
const bodyY = cy - bodyH / 2;
const bodyW = w - bodyX - lineWidth / 2;
const bodyCx = bodyX + bodyW / 2;
const bodyRadius = bodyH * 0.3;

ctx.fillStyle = color;
roundedRectPath(capX, capY, capW, capH, capH * 0.35);
ctx.fill();

ctx.strokeStyle = color;
ctx.lineWidth = lineWidth;
roundedRectPath(bodyX, bodyY, bodyW, bodyH, bodyRadius);
ctx.stroke();

// fillable inner area, with extra padding away from the outline
const padding = lineWidth * 1.7;
const innerX = bodyX + padding;
const innerY = bodyY + padding;
const innerW = bodyW - padding * 2;
const innerH = bodyH - padding * 2;
const innerRadius = Math.max(innerH * 0.25, 1);

if (hasBattery) {
  const fillRatio = Math.min(Math.max(percentage, 0), 100) / 100;
  const fillW = Math.max(innerW * fillRatio, innerH);
  ctx.fillStyle = accent;
  roundedRectPath(innerX, innerY, fillW, innerH, Math.min(innerRadius, fillW / 2));
  ctx.fill();
}

function drawLightning() {
  const boltW = innerH * 0.5;
  const boltH = innerH * 1.05;
  const top = cy - boltH / 2;
  const bottom = cy + boltH / 2;

  ctx.beginPath();
  ctx.moveTo(bodyCx + boltW * 0.18, top);
  ctx.lineTo(bodyCx - boltW * 0.42, cy + boltH * 0.05);
  ctx.lineTo(bodyCx, cy + boltH * 0.05);
  ctx.lineTo(bodyCx - boltW * 0.18, bottom);
  ctx.lineTo(bodyCx + boltW * 0.42, cy - boltH * 0.05);
  ctx.lineTo(bodyCx, cy - boltH * 0.05);
  ctx.closePath();
  ctx.fillStyle = color;
  ctx.fill();
}

function drawDanger() {
  const size = innerH * 0.95;
  const half = size * 0.58;
  const top = cy - size * 0.55;
  const bottom = cy + size * 0.45;

  ctx.beginPath();
  ctx.moveTo(bodyCx, top);
  ctx.lineTo(bodyCx + half, bottom);
  ctx.lineTo(bodyCx - half, bottom);
  ctx.closePath();
  ctx.fillStyle = dangerColor;
  ctx.fill();

  ctx.fillStyle = themeTokens.backgroundColor;
  const barW = size * 0.12;
  ctx.fillRect(bodyCx - barW / 2, top + size * 0.3, barW, size * 0.3);
  ctx.fillRect(bodyCx - barW / 2, top + size * 0.72, barW, barW);
}

if (!hasBattery) {
  drawDanger();
} else if (isCharging) {
  drawLightning();
}
