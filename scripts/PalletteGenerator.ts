// generate-oklch-scale.ts

function oklchToP3(L: number, C: number, H: number) {
  const hRad = (H * Math.PI) / 180;

  const a = C * Math.cos(hRad);
  const b = C * Math.sin(hRad);

  const l = L + 0.3963377774 * a + 0.2158037573 * b;
  const m = L - 0.1055613458 * a - 0.0638541728 * b;
  const s = L - 0.0894841775 * a - 1.291485548 * b;

  const l3 = l * l * l;
  const m3 = m * m * m;
  const s3 = s * s * s;

  // OKLab → XYZ
  const X = 1.2270138511 * l3 - 0.5577999807 * m3 + 0.281256149 * s3;
  const Y = -0.0405801784 * l3 + 1.1122568696 * m3 - 0.0716766787 * s3;
  const Z = -0.0763812845 * l3 - 0.4214819784 * m3 + 1.5861632204 * s3;

  // XYZ → Display-P3
  return {
    r: 2.493496911941425 * X - 0.9313836179191239 * Y - 0.40271078445071684 * Z,
    g: -0.8294889695615747 * X + 1.7626640603183463 * Y + 0.023624685841943577 * Z,
    b: 0.03584583024378447 * X - 0.07617238926804182 * Y + 0.9568845240076872 * Z,
  };
}

function inP3(r: number, g: number, b: number) {
  const eps = 1e-5;
  return r >= -eps && r <= 1 + eps && g >= -eps && g <= 1 + eps && b >= -eps && b <= 1 + eps;
}

function maxChromaP3(L: number, H: number) {
  let low = 0;
  let high = 1;

  for (let i = 0; i < 30; i++) {
    const mid = (low + high) / 2;
    const { r, g, b } = oklchToP3(L, mid, H);

    if (inP3(r, g, b)) low = mid;
    else high = mid;
  }

  return low;
}

function oklchToSRGB(L: number, C: number, H: number) {
  const hRad = (H * Math.PI) / 180;

  const a = C * Math.cos(hRad);
  const b = C * Math.sin(hRad);

  const l = L + 0.3963377774 * a + 0.2158037573 * b;
  const m = L - 0.1055613458 * a - 0.0638541728 * b;
  const s = L - 0.0894841775 * a - 1.291485548 * b;

  const l3 = l * l * l;
  const m3 = m * m * m;
  const s3 = s * s * s;

  const r = 4.0767416621 * l3 + -3.3077115913 * m3 + 0.2309699292 * s3;

  const g = -1.2684380046 * l3 + 2.6097574011 * m3 + -0.3413193965 * s3;

  const b2 = -0.0041960863 * l3 + -0.7034186147 * m3 + 1.707614701 * s3;

  return { r, g, b: b2 };
}

function inSRGB(r: number, g: number, b: number) {
  return r >= 0 && r <= 1 && g >= 0 && g <= 1 && b >= 0 && b <= 1;
}

function _maxChromaSRGB(L: number, H: number) {
  let low = 0;
  let high = 0.4;

  for (let i = 0; i < 30; i++) {
    const mid = (low + high) / 2;

    const { r, g, b } = oklchToSRGB(L, mid, H);

    if (inSRGB(r, g, b)) {
      low = mid;
    } else {
      high = mid;
    }
  }

  return low;
}

const LIGHT_STEPS = {
  25: 99,
  50: 96,
  75: 93,
  100: 90,
  200: 81,
  300: 72,
  400: 63,
  500: 54,
  600: 45,
  700: 36,
  800: 27,
  900: 18,
};

const DARK_STEPS = {
  25: 9,
  50: 12,
  75: 15,
  100: 18,
  200: 27,
  300: 36,
  400: 45,
  500: 54,
  600: 63,
  700: 72,
  800: 81,
  900: 90,
};

const COLOR_LIST = {
  gray: 0,
  red: 30,
  orange: 50,
  yellow: 100,
  green: 150,
  seafoam: 180,
  cyan: 230,
  blue: 260,
  indigo: 280,
  purple: 300,
  fuchsia: 335,
  magenta: 360,
};

function generateScale(name: string, hue: number, steps: Record<number, number>) {
  const lines: string[] = [];

  for (const [step, lightness] of Object.entries(steps)) {
    const L = lightness / 100;

    const C = hue === 0 ? 0 : maxChromaP3(L, hue) * 0.8;

    const truncatedC = truncate(C, 3).toString().padEnd(5, "0");

    lines.push(`    --color-${name}-${step}: oklch(${lightness}% ${truncatedC} ${hue});`);
  }

  return lines.join("\n");
}

function generateScheme(scheme: "light" | "dark" | "none", steps: Record<number, number>) {
  const lines: string[] = [];
  const hasScheme = scheme !== "none";

  if (hasScheme) {
    lines.push(`@media (prefers-color-scheme: ${scheme}) {`);
  }
  lines.push(`  :root {`);

  let groups: string[] = [];
  for (const [name, hue] of Object.entries(COLOR_LIST)) {
    groups.push(generateScale(hasScheme ? name : `fixed-${name}`, hue, steps));
  }

  lines.push(groups.join("\n\n"));
  lines.push("  }");
  if (hasScheme) {
    lines.push("}");
  }

  return lines.join("\n");
}

function truncate(num: number, decimals: number): number {
  const factor = 10 ** decimals;
  return Math.trunc(num * factor) / factor;
}

console.log(generateScheme("dark", DARK_STEPS));
console.log("");
console.log(generateScheme("light", LIGHT_STEPS));
console.log("");
console.log(generateScheme("none", LIGHT_STEPS));
