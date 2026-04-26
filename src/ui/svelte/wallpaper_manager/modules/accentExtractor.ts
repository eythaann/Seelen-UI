import type { Color } from "@seelen-ui/lib/types";

const CANVAS_SIZE = 64;
const K = 6;
const ITERATIONS = 20;
// Only sample the center region to avoid irrelevant sky/floor bands
const CENTER_RATIO = 0.6;

type HSV = [number, number, number]; // h ∈ [0,1), s ∈ [0,1], v ∈ [0,1]

function rgbToHsv(r: number, g: number, b: number): HSV {
  const rn = r / 255,
    gn = g / 255,
    bn = b / 255;
  const max = Math.max(rn, gn, bn),
    min = Math.min(rn, gn, bn),
    d = max - min;
  let h = 0;
  const s = max === 0 ? 0 : d / max;
  const v = max;
  if (d !== 0) {
    if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6;
    else if (max === gn) h = ((bn - rn) / d + 2) / 6;
    else h = ((rn - gn) / d + 4) / 6;
  }
  return [h, s, v];
}

function hsvToRgb(h: number, s: number, v: number): [number, number, number] {
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s),
    q = v * (1 - f * s),
    t = v * (1 - (1 - f) * s);
  let r = 0,
    g = 0,
    b = 0;
  switch (i % 6) {
    case 0:
      [r, g, b] = [v, t, p];
      break;
    case 1:
      [r, g, b] = [q, v, p];
      break;
    case 2:
      [r, g, b] = [p, v, t];
      break;
    case 3:
      [r, g, b] = [p, q, v];
      break;
    case 4:
      [r, g, b] = [t, p, v];
      break;
    case 5:
      [r, g, b] = [v, p, q];
      break;
  }
  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

// Euclidean distance in HSV with circular hue
function hsvDistance(a: HSV, b: HSV): number {
  const dh = Math.min(Math.abs(a[0] - b[0]), 1 - Math.abs(a[0] - b[0])) * 2;
  const ds = a[1] - b[1];
  const dv = a[2] - b[2];
  return dh * dh + ds * ds + dv * dv;
}

function kMeans(pixels: HSV[], k: number, iterations: number): HSV[] {
  if (pixels.length === 0) return [];

  // K-means++ initialization for better coverage
  const centroids: HSV[] = [pixels[Math.floor(Math.random() * pixels.length)]!];
  while (centroids.length < k) {
    const weights = pixels.map((p) => Math.min(...centroids.map((c) => hsvDistance(p, c))));
    const total = weights.reduce((s, w) => s + w, 0);
    let r = Math.random() * total;
    let pushed = false;
    for (let i = 0; i < pixels.length; i++) {
      r -= weights[i]!;
      if (r <= 0) {
        centroids.push(pixels[i]!);
        pushed = true;
        break;
      }
    }
    if (!pushed) centroids.push(pixels[pixels.length - 1]!);
  }

  for (let iter = 0; iter < iterations; iter++) {
    const clusters: HSV[][] = Array.from({ length: k }, () => []);
    for (const pixel of pixels) {
      let minDist = Infinity,
        best = 0;
      for (let i = 0; i < k; i++) {
        const d = hsvDistance(pixel, centroids[i]!);
        if (d < minDist) {
          minDist = d;
          best = i;
        }
      }
      clusters[best]!.push(pixel);
    }
    for (let i = 0; i < k; i++) {
      const cluster = clusters[i]!;
      if (cluster.length === 0) continue;
      // Circular mean for hue, arithmetic for s and v
      let sinSum = 0,
        cosSum = 0,
        sSum = 0,
        vSum = 0,
        wSum = 0;
      for (const [h, s, v] of cluster) {
        const w = s * s;
        sinSum += Math.sin(h * 2 * Math.PI) * w;
        cosSum += Math.cos(h * 2 * Math.PI) * w;
        sSum += s * w;
        vSum += v * w;
        wSum += w;
      }
      const n = wSum > 0 ? wSum : cluster.length;
      const fallback = wSum === 0;
      if (fallback) {
        for (const [h, s, v] of cluster) {
          sinSum += Math.sin(h * 2 * Math.PI);
          cosSum += Math.cos(h * 2 * Math.PI);
          sSum += s;
          vSum += v;
        }
      }
      centroids[i] = [
        (Math.atan2(sinSum / n, cosSum / n) / (2 * Math.PI) + 1) % 1,
        sSum / n,
        vSum / n,
      ];
    }
  }

  return centroids;
}

const MIN_SATURATION = 0.2;
// Minimum fraction of pixels that must be chromatic before using the filtered set
const MIN_CHROMATIC_RATIO = 0.05;

function samplePixels(imageData: ImageData): HSV[] {
  const { data, width, height } = imageData;
  const margin = Math.floor(((1 - CENTER_RATIO) / 2) * Math.min(width, height));
  const all: HSV[] = [];
  const chromatic: HSV[] = [];

  for (let y = margin; y < height - margin; y++) {
    for (let x = margin; x < width - margin; x++) {
      const i = (y * width + x) * 4;
      if (data[i + 3]! < 128) continue;
      const hsv = rgbToHsv(data[i]!, data[i + 1]!, data[i + 2]!);
      all.push(hsv);
      // Exclude grays and near-black pixels so they don't pull vivid cluster centroids toward gray
      if (hsv[1] >= MIN_SATURATION && hsv[2] >= 0.1) chromatic.push(hsv);
    }
  }

  // Use the chromatic-only set when it's representative enough
  return chromatic.length >= all.length * MIN_CHROMATIC_RATIO && chromatic.length >= K ? chromatic : all;
}

function pickBestColor(centroids: HSV[]): Color {
  // Discard near-black, near-white, and unsaturated centroids
  const filtered = centroids.filter(([, s, v]) => v >= 0.2 && v <= 0.92 && s >= 0.2);
  const candidates = filtered.length > 0 ? filtered : [...centroids].sort((a, b) => b[1] - a[1]);

  // Prefer vivid colors: high saturation + brightness close to 0.65 (sweet spot)
  const scored = candidates.map((hsv) => {
    const [, s, v] = hsv;
    const vibrance = s * (1 - Math.abs(v - 0.65) * 0.8);
    return { hsv, score: vibrance };
  });
  scored.sort((a, b) => b.score - a.score);

  const [h, s, v] = scored[0]!.hsv;
  const [r, g, b] = hsvToRgb(h, s, v);
  return { r, g, b, a: 255 };
}

export function extractAccentColor(
  element: HTMLImageElement | HTMLVideoElement,
): Color | null {
  const canvas = document.createElement("canvas");
  canvas.width = CANVAS_SIZE;
  canvas.height = CANVAS_SIZE;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) return null;

  // Nearest-neighbor prevents bilinear blending from dulling vivid colors
  // (e.g. red pixels adjacent to black would average to dark red with smoothing on)
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(element, 0, 0, CANVAS_SIZE, CANVAS_SIZE);
  const imageData = ctx.getImageData(0, 0, CANVAS_SIZE, CANVAS_SIZE);
  const pixels = samplePixels(imageData);
  if (pixels.length === 0) return null;
  const centroids = kMeans(pixels, K, ITERATIONS);
  if (centroids.length === 0) return null;
  return pickBestColor(centroids);
}
