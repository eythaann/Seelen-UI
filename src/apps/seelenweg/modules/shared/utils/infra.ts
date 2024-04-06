import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

import { store } from '../store/infra';

import { HWND, UWP_Package } from '../store/domain';

export const LAZY_CONSTANTS = {
  MISSING_ICON_PATH: '',
  TEMP_FOLDER: '',
};

export async function loadConstants() {
  LAZY_CONSTANTS.MISSING_ICON_PATH = await getMissingIconPath();
  LAZY_CONSTANTS.TEMP_FOLDER = await path.tempDir();
}

export async function getMissingIconPath() {
  return await path.resolve(
    await path.resourceDir(),
    'static',
    'icons',
    'missing.png',
  );
}

export async function updatePreviews(hwnds: HWND[]) {
  const state = store.getState();
  const process = hwnds.map((hwnd) => {
    return {
      hwnd,
      process_hwnd: state.openApps[hwnd]?.process_hwnd || (0 as HWND),
    };
  });
  invoke('weg_request_update_previews', { hwnds: process });
}

export async function iconPathFromExePath(exePath: string) {
  const parts = exePath.split('\\');
  const fileName = parts.at(-1)?.replace('.exe', '.png') || 'missing.png';
  return await path.resolve(await path.resourceDir(), 'gen', 'icons', fileName);
}

/**
 * For some reason uwp_manifests.json can no be readed and parsed by JSON.parse()
 * so I use fetch as solution, maybe is a problem with the encoding of the file
 */
export async function getUWPInfoFromExePath(exePath: string): Promise<UWP_Package | undefined> {
  if (!exePath) {
    return undefined;
  }
  const dirname = await path.dirname(exePath);
  const url = convertFileSrc(await path.resolveResource('gen/uwp_manifests.json'));
  const response = await fetch(url);
  const manifests: UWP_Package[] = await response.json();
  return manifests.find((manifest) => manifest.InstallLocation === dirname);
}

export function getImageBase64FromUrl(url: string): Promise<string> {
  let imagen = new Image();
  let canvas = document.createElement('canvas');
  let ctx = canvas.getContext('2d')!;

  return new Promise((resolve, reject) => {
    imagen.onload = function () {
      canvas.width = imagen.width;
      canvas.height = imagen.height;
      ctx.drawImage(imagen, 0, 0);
      resolve(trimCanvas(canvas).toDataURL());
    };

    imagen.onerror = function () {
      console.error('Error while loading image: ', url);
      reject();
    };

    imagen.crossOrigin = '';
    imagen.src = url;
  });
}

function trimCanvas(canvas: HTMLCanvasElement) {
  function rowBlank(imageData: ImageData, width: number, y: number) {
    for (let x = 0; x < width; ++x) {
      if (imageData.data[y * width * 4 + x * 4 + 3] !== 0) return false;
    }
    return true;
  }

  function columnBlank(
    imageData: ImageData,
    width: number,
    x: number,
    top: number,
    bottom: number,
  ) {
    for (let y = top; y < bottom; ++y) {
      if (imageData.data[y * width * 4 + x * 4 + 3] !== 0) return false;
    }
    return true;
  }

  canvas.setAttribute;
  const ctx = canvas.getContext('2d', { willReadFrequently: true })!;
  const width = canvas.width;
  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
  let top = 0,
    bottom = imageData.height,
    left = 0,
    right = imageData.width;

  while (top < bottom && rowBlank(imageData, width, top)) ++top;
  while (bottom - 1 > top && rowBlank(imageData, width, bottom - 1)) --bottom;
  while (left < right && columnBlank(imageData, width, left, top, bottom)) ++left;
  while (right - 1 > left && columnBlank(imageData, width, right - 1, top, bottom)) --right;

  const trimmed = ctx.getImageData(left, top, right - left, bottom - top);
  const copy = canvas.ownerDocument.createElement('canvas');
  const copyCtx = copy.getContext('2d', { willReadFrequently: true })!;
  copy.width = trimmed.width;
  copy.height = trimmed.height;
  copyCtx.putImageData(trimmed, 0, 0);

  return copy;
}

export async function isDevMode() {
  return await invoke<boolean>('is_dev_mode');
}
