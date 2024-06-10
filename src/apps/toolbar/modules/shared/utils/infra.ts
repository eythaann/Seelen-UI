import { path } from '@tauri-apps/api';

export const LAZY_CONSTANTS = {
  MISSING_ICON_PATH: '',
};

export async function loadConstants() {
  LAZY_CONSTANTS.MISSING_ICON_PATH = await getMissingIconPath();
}

export async function getMissingIconPath() {
  return await path.resolve(await path.resourceDir(), 'static', 'icons', 'missing.png');
}