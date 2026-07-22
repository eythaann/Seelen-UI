const ctx = canvas.getContext("2d");
if (!ctx) return;

const now = new Date();

const w = canvas.width;
const h = canvas.height;
const cx = w / 2;
const cy = h / 2;

const lineWidth = w * 0.06;
const radius = Math.min(w, h) / 2 - lineWidth / 2;

ctx.clearRect(0, 0, w, h);

ctx.lineCap = "round";
ctx.lineJoin = "round";

// Clock outline
ctx.beginPath();
ctx.arc(cx, cy, radius, 0, Math.PI * 2);
ctx.lineWidth = lineWidth;
ctx.strokeStyle = themeTokens.foregroundColor;
ctx.stroke();

const seconds = now.getSeconds();
const minutes = now.getMinutes();
const hours = now.getHours() % 12;

// const secondsAngle = (seconds / 60) * Math.PI * 2 - Math.PI / 2;
const minutesAngle = ((minutes + seconds / 60) / 60) * Math.PI * 2 - Math.PI / 2;
const hoursAngle = ((hours + minutes / 60) / 12) * Math.PI * 2 - Math.PI / 2;

function drawHand(angle, length, width) {
  ctx.beginPath();
  ctx.moveTo(cx, cy);
  ctx.lineTo(cx + Math.cos(angle) * length, cy + Math.sin(angle) * length);
  ctx.lineWidth = width;
  ctx.strokeStyle = themeTokens.foregroundColor;
  ctx.stroke();
}

// Clock hands
drawHand(hoursAngle, radius * 0.5, lineWidth / 1.5);
drawHand(minutesAngle, radius * 0.75, lineWidth / 2);

// Center cap
ctx.beginPath();
ctx.arc(cx, cy, lineWidth, 0, Math.PI * 2);
ctx.fillStyle = themeTokens.foregroundColor;
ctx.fill();
