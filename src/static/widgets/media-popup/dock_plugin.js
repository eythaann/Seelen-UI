const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cy = h / 2;

ctx.clearRect(0, 0, w, h);
ctx.lineCap = "round";
ctx.lineJoin = "round";

const level = isMuted ? 0 : volume;
const color = themeTokens.foregroundColor;
const mutedColor = themeTokens.foregroundMutedColor;
const lineWidth = w * 0.06;

// speaker body: rectangle base + triangle cone, occupying the left half
const bodyW = w * 0.2;
const bodyH = h * 0.34;
const bodyX = w * 0.06;
const bodyY = cy - bodyH / 2;

const coneTipX = w * 0.5;
const coneH = h * 0.62;

function roundedPolygonPath(points, radius) {
  ctx.beginPath();
  const n = points.length;
  const start = points[0];
  const prev = points[n - 1];
  ctx.moveTo((prev.x + start.x) / 2, (prev.y + start.y) / 2);
  for (let i = 0; i < n; i++) {
    const p = points[i];
    const next = points[(i + 1) % n];
    ctx.arcTo(p.x, p.y, next.x, next.y, radius);
  }
  ctx.closePath();
}

roundedPolygonPath(
  [
    { x: bodyX, y: bodyY },
    { x: bodyX + bodyW, y: bodyY },
    { x: coneTipX, y: cy - coneH / 2 },
    { x: coneTipX, y: cy + coneH / 2 },
    { x: bodyX + bodyW, y: bodyY + bodyH },
    { x: bodyX, y: bodyY + bodyH },
  ],
  w * 0.035,
);
ctx.fillStyle = color;
ctx.fill();

function drawWaves() {
  const arcs = 3;
  const centerX = coneTipX - w * 0.02;
  const minRadius = w * 0.14;
  const maxRadius = w - centerX - lineWidth / 2;
  const step = (maxRadius - minRadius) / (arcs - 1);
  ctx.lineWidth = lineWidth;
  for (let i = 1; i <= arcs; i++) {
    ctx.strokeStyle = level >= i / (arcs + 1) ? color : mutedColor;
    ctx.beginPath();
    ctx.arc(centerX, cy, minRadius + step * (i - 1), -Math.PI * 0.24, Math.PI * 0.24);
    ctx.stroke();
  }
}

function drawSlash() {
  ctx.beginPath();
  ctx.moveTo(w * 0.95, h * 0.05);
  ctx.lineTo(w * 0.05, h * 0.95);
  ctx.lineWidth = lineWidth;
  ctx.strokeStyle = mutedColor;
  ctx.stroke();
}

drawWaves();
if (isMuted) {
  drawSlash();
}
