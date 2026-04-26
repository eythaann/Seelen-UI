import type { Color } from "@seelen-ui/lib/types";

const CANVAS_SIZE = 64;
const CLUSTER_COUNT = 6;
const KMEANS_ITERATIONS = 20;

// Only sample the center region to avoid sky/floor bias
const CENTER_SAMPLE_RATIO = 0.6;

// Filtering thresholds (tuned for UI accent extraction)
const MIN_SATURATION = 0.25;
const MIN_BRIGHTNESS = 0.25;

type HSVColor = [number, number, number]; // [hue, saturation, value]

// ----------------------
// Color space conversion
// ----------------------

function convertRgbToHsv(red: number, green: number, blue: number): HSVColor {
  const redNorm = red / 255;
  const greenNorm = green / 255;
  const blueNorm = blue / 255;

  const maxChannel = Math.max(redNorm, greenNorm, blueNorm);
  const minChannel = Math.min(redNorm, greenNorm, blueNorm);
  const delta = maxChannel - minChannel;

  let hue = 0;
  const saturation = maxChannel === 0 ? 0 : delta / maxChannel;
  const value = maxChannel;

  if (delta !== 0) {
    if (maxChannel === redNorm) {
      hue = ((greenNorm - blueNorm) / delta + (greenNorm < blueNorm ? 6 : 0)) / 6;
    } else if (maxChannel === greenNorm) {
      hue = ((blueNorm - redNorm) / delta + 2) / 6;
    } else {
      hue = ((redNorm - greenNorm) / delta + 4) / 6;
    }
  }

  return [hue, saturation, value];
}

function convertHsvToRgb(hue: number, saturation: number, value: number): [number, number, number] {
  const sector = Math.floor(hue * 6);
  const fraction = hue * 6 - sector;

  const p = value * (1 - saturation);
  const q = value * (1 - fraction * saturation);
  const t = value * (1 - (1 - fraction) * saturation);

  let red = 0;
  let green = 0;
  let blue = 0;

  switch (sector % 6) {
    case 0:
      [red, green, blue] = [value, t, p];
      break;
    case 1:
      [red, green, blue] = [q, value, p];
      break;
    case 2:
      [red, green, blue] = [p, value, t];
      break;
    case 3:
      [red, green, blue] = [p, q, value];
      break;
    case 4:
      [red, green, blue] = [t, p, value];
      break;
    case 5:
      [red, green, blue] = [value, p, q];
      break;
  }

  return [Math.round(red * 255), Math.round(green * 255), Math.round(blue * 255)];
}

// --------------------------------------
// HSV distance (with circular hue logic)
// --------------------------------------

function computeHsvDistance(colorA: HSVColor, colorB: HSVColor): number {
  const hueDistance = Math.min(Math.abs(colorA[0] - colorB[0]), 1 - Math.abs(colorA[0] - colorB[0])) * 2;

  const saturationDistance = colorA[1] - colorB[1];
  const valueDistance = colorA[2] - colorB[2];

  return (
    hueDistance * hueDistance +
    saturationDistance * saturationDistance +
    valueDistance * valueDistance
  );
}

// ----------------------
// Pixel sampling (clean)
// ----------------------

function sampleRelevantPixels(imageData: ImageData): HSVColor[] {
  const { data, width, height } = imageData;

  const margin = Math.floor(((1 - CENTER_SAMPLE_RATIO) / 2) * Math.min(width, height));

  const sampledPixels: HSVColor[] = [];

  for (let y = margin; y < height - margin; y++) {
    for (let x = margin; x < width - margin; x++) {
      const pixelIndex = (y * width + x) * 4;

      const alpha = data[pixelIndex + 3] || 0;
      if (alpha < 128) continue;

      const red = data[pixelIndex] || 0;
      const green = data[pixelIndex + 1] || 0;
      const blue = data[pixelIndex + 2] || 0;

      const hsvColor = convertRgbToHsv(red, green, blue);

      // 🔥 Important: remove low-saturation (gray) and dark pixels early
      if (hsvColor[1] < MIN_SATURATION) continue;
      if (hsvColor[2] < MIN_BRIGHTNESS) continue;

      sampledPixels.push(hsvColor);
    }
  }

  return sampledPixels.length >= CLUSTER_COUNT ? sampledPixels : [];
}

// ----------------------
// K-means clustering
// ----------------------

function performKMeansClustering(
  pixels: HSVColor[],
  clusterCount: number,
  iterations: number,
): HSVColor[] {
  if (pixels.length === 0) return [];

  // --- K-means++ initialization ---
  const centroids: HSVColor[] = [pixels[Math.floor(Math.random() * pixels.length)]!];

  while (centroids.length < clusterCount) {
    const distances = pixels.map((pixel) =>
      Math.min(...centroids.map((centroid) => computeHsvDistance(pixel, centroid)))
    );

    const totalDistance = distances.reduce((sum, value) => sum + value, 0);
    let randomValue = Math.random() * totalDistance;

    for (let i = 0; i < pixels.length; i++) {
      randomValue -= distances[i]!;
      if (randomValue <= 0) {
        centroids.push(pixels[i]!);
        break;
      }
    }
  }

  // --- Iterative refinement ---
  for (let iteration = 0; iteration < iterations; iteration++) {
    const clusters: HSVColor[][] = Array.from({ length: clusterCount }, () => []);

    // Assign pixels to closest centroid
    for (const pixel of pixels) {
      let closestIndex = 0;
      let smallestDistance = Infinity;

      for (let i = 0; i < centroids.length; i++) {
        const distance = computeHsvDistance(pixel, centroids[i]!);
        if (distance < smallestDistance) {
          smallestDistance = distance;
          closestIndex = i;
        }
      }

      clusters[closestIndex]!.push(pixel);
    }

    // Recompute centroids
    for (let i = 0; i < clusterCount; i++) {
      const cluster = clusters[i]!;
      if (cluster.length === 0) continue;

      let sineSum = 0;
      let cosineSum = 0;
      let saturationSum = 0;
      let weightSum = 0;

      const valueSamples: number[] = [];

      for (const [hue, saturation, value] of cluster) {
        const weight = saturation * saturation;

        sineSum += Math.sin(hue * 2 * Math.PI) * weight;
        cosineSum += Math.cos(hue * 2 * Math.PI) * weight;

        saturationSum += saturation * weight;
        weightSum += weight;

        valueSamples.push(value);
      }

      const normalizedWeight = weightSum > 0 ? weightSum : cluster.length;

      // Circular mean for hue
      const averagedHue = (Math.atan2(sineSum / normalizedWeight, cosineSum / normalizedWeight) / (2 * Math.PI) + 1) %
        1;

      const averagedSaturation = saturationSum / normalizedWeight;

      // 🔥 Key improvement:
      // Use a high percentile instead of average to preserve brightness
      valueSamples.sort((a, b) => a - b);
      const percentileIndex = Math.floor(valueSamples.length * 0.8);
      const representativeValue = valueSamples[percentileIndex]!;

      centroids[i] = [averagedHue, averagedSaturation, representativeValue];
    }
  }

  return centroids;
}

// ----------------------
// Accent selection logic
// ----------------------

function selectBestAccentColor(centroids: HSVColor[]): Color {
  const filteredCentroids = centroids.filter(
    ([, saturation, value]) => saturation >= 0.25 && value >= 0.3,
  );

  const candidateCentroids = filteredCentroids.length > 0 ? filteredCentroids : centroids;

  const scoredCentroids = candidateCentroids.map((hsvColor) => {
    const [, saturation, value] = hsvColor;

    // 🔥 Favor vivid + bright colors
    const score = saturation * Math.pow(value, 1.2);

    return { hsvColor, score };
  });

  scoredCentroids.sort((a, b) => b.score - a.score);

  let [hue, saturation, value] = scoredCentroids[0]!.hsvColor;

  // ✨ Small boost to make UI accents feel more alive
  saturation = Math.min(1, saturation * 1.1);
  value = Math.min(1, value * 1.1);

  const [red, green, blue] = convertHsvToRgb(hue, saturation, value);

  return { r: red, g: green, b: blue, a: 255 };
}

// ----------------------
// Public API
// ----------------------

export function extractAccentColorFromElement(
  element: HTMLImageElement | HTMLVideoElement,
): Color | null {
  const canvas = document.createElement("canvas");
  canvas.width = CANVAS_SIZE;
  canvas.height = CANVAS_SIZE;

  const context = canvas.getContext("2d", {
    willReadFrequently: true,
  });

  if (!context) return null;

  // Disable smoothing to avoid blending colors (important!)
  context.imageSmoothingEnabled = false;

  context.drawImage(element, 0, 0, CANVAS_SIZE, CANVAS_SIZE);

  const imageData = context.getImageData(0, 0, CANVAS_SIZE, CANVAS_SIZE);

  const sampledPixels = sampleRelevantPixels(imageData);
  if (sampledPixels.length === 0) return null;

  const centroids = performKMeansClustering(sampledPixels, CLUSTER_COUNT, KMEANS_ITERATIONS);

  if (centroids.length === 0) return null;

  return selectBestAccentColor(centroids);
}

export function extractAccentColorFromSrc(source: string): Promise<Color | null> {
  return new Promise((resolve) => {
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.onload = () => {
      resolve(extractAccentColorFromElement(image));
    };
    image.onerror = () => resolve(null);
    image.src = source;
  });
}
