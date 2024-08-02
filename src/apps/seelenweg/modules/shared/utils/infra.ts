import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';

import { store } from '../store/infra';

import { getGeneratedFilesPath } from './app';

import { HWND } from '../store/domain';

export const LAZY_CONSTANTS = {
  MISSING_ICON_PATH: '',
  DEFAULT_THUMBNAIL: '',
  TEMP_FOLDER: '',
};

export async function loadConstants() {
  LAZY_CONSTANTS.TEMP_FOLDER = await path.tempDir();
  LAZY_CONSTANTS.MISSING_ICON_PATH = await getMissingIconPath();
  LAZY_CONSTANTS.DEFAULT_THUMBNAIL = await path.resolve(
    await path.resourceDir(),
    'static',
    'icons',
    'default_thumbnail.jpg',
  );
}

export async function getMissingIconPath() {
  return await path.resolve(await path.resourceDir(), 'static', 'icons', 'missing.png');
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
  return await path.resolve(await getGeneratedFilesPath(), 'icons', fileName);
}

export function getImageBase64FromUrl(url: string): Promise<string> {
  let image = new Image();
  let canvas = document.createElement('canvas');
  let ctx = canvas.getContext('2d')!;

  return new Promise((resolve, reject) => {
    image.onload = function () {
      canvas.width = image.width;
      canvas.height = image.height;
      ctx.drawImage(image, 0, 0);
      resolve(trimCanvas(canvas).toDataURL());
    };

    image.onerror = function () {
      console.error('Error while loading image: ', url);
      reject();
    };

    image.crossOrigin = '';
    image.src = url;
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
