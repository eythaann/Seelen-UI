import { path } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/core';

export const Constants = {
  MISSING_ICON: '',
  TEMP_FOLDER: '',
};

export async function loadConstants() {
  Constants.MISSING_ICON = await getMissingIconUrl();
  Constants.TEMP_FOLDER = await path.tempDir();
}

export async function getMissingIconUrl() {
  const missingIcon = await path.resolve(
    await path.resourceDir(),
    'static',
    'icons',
    'missing.png',
  );
  return convertFileSrc(missingIcon);
}