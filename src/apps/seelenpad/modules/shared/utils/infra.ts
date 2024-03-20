import { path } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/core';

/** relative path to resources/sounds */
export async function playSound(resourcePath: string, volume = 0.1) {
  const audioPath = await path.resolveResource('static/sounds/' + resourcePath);
  const assetUrl = convertFileSrc(audioPath);
  const audio = new Audio(assetUrl);
  audio.volume = volume;
  audio.play();
}