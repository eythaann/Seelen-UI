const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cx = w / 2;

ctx.clearRect(0, 0, w, h);
ctx.lineCap = "round";
ctx.lineJoin = "round";

const isWifi = usingInterface?.type === "IEEE80211";
const color = online ? themeTokens.foregroundColor : themeTokens.foregroundMutedColor;
const lineWidth = w * 0.08;

function drawWifi() {
  const baseY = h * 0.78;
  const dotRadius = lineWidth * 0.6;

  ctx.beginPath();
  ctx.arc(cx, baseY, dotRadius, 0, Math.PI * 2);
  ctx.fillStyle = color;
  ctx.fill();

  const arcs = 3;
  const spacing = (h * 0.62) / arcs;
  ctx.strokeStyle = color;
  ctx.lineWidth = lineWidth;
  for (let i = 1; i <= arcs; i++) {
    const radius = spacing * i;
    ctx.beginPath();
    ctx.arc(cx, baseY, radius, Math.PI * 1.25, Math.PI * 1.75);
    ctx.stroke();
  }
}

function drawSlash() {
  ctx.beginPath();
  ctx.moveTo(w * 0.18, h * 0.18);
  ctx.lineTo(w * 0.82, h * 0.82);
  ctx.lineWidth = lineWidth * 1.1;
  ctx.strokeStyle = color;
  ctx.stroke();
}

function drawEthernet() {
  const cableTop = h * 0.18;
  const cableBottom = h * 0.55;
  const plugWidth = w * 0.32;
  const plugHeight = h * 0.28;

  ctx.strokeStyle = color;
  ctx.lineWidth = lineWidth;

  // cable
  ctx.beginPath();
  ctx.moveTo(cx, cableTop);
  ctx.lineTo(cx, cableBottom);
  ctx.stroke();

  // connector body
  ctx.strokeRect(cx - plugWidth / 2, cableBottom, plugWidth, plugHeight);

  // pins
  const pinCount = 3;
  const pinSpacing = plugWidth / (pinCount + 1);
  for (let i = 1; i <= pinCount; i++) {
    const x = cx - plugWidth / 2 + pinSpacing * i;
    ctx.beginPath();
    ctx.moveTo(x, cableBottom + plugHeight);
    ctx.lineTo(x, cableBottom + plugHeight + lineWidth * 0.8);
    ctx.stroke();
  }
}

if (!online) {
  drawWifi();
  drawSlash();
} else if (isWifi) {
  drawWifi();
} else {
  drawEthernet();
}
