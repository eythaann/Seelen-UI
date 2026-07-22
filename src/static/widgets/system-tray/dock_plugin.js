const ctx = canvas.getContext("2d");
if (!ctx) return;

const w = canvas.width;
const h = canvas.height;
const cx = w / 2;
const cy = h / 2;

ctx.clearRect(0, 0, w, h);
ctx.lineCap = "round";
ctx.lineJoin = "round";

const color = themeTokens.foregroundColor;
const size = w * 0.22;
const lineWidth = w * 0.085;

ctx.strokeStyle = color;
ctx.lineWidth = lineWidth;

// the popup opens away from the screen edge the dock is docked to,
// so the chevron points in that same direction.
ctx.beginPath();
switch (position) {
  case "Top":
    ctx.moveTo(cx - size, cy - size / 2);
    ctx.lineTo(cx, cy + size / 2);
    ctx.lineTo(cx + size, cy - size / 2);
    break;
  case "Left":
    ctx.moveTo(cx - size / 2, cy - size);
    ctx.lineTo(cx + size / 2, cy);
    ctx.lineTo(cx - size / 2, cy + size);
    break;
  case "Right":
    ctx.moveTo(cx + size / 2, cy - size);
    ctx.lineTo(cx - size / 2, cy);
    ctx.lineTo(cx + size / 2, cy + size);
    break;
  case "Bottom":
  default:
    ctx.moveTo(cx - size, cy + size / 2);
    ctx.lineTo(cx, cy - size / 2);
    ctx.lineTo(cx + size, cy + size / 2);
    break;
}
ctx.stroke();
