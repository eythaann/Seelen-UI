import { SeelenCommand } from '@seelen-ui/lib';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';

import { getGeneratedFilesPath } from './app';

import { HWND } from '../store/domain';

export const LAZY_CONSTANTS = {
  FOLDER_ICON_PATH: '',
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
    'music_thumbnail.jpg',
  );
  LAZY_CONSTANTS.FOLDER_ICON_PATH = await path.resolve(
    await path.resourceDir(),
    'static',
    'icons',
    'folder.svg',
  );
}

export async function getMissingIconPath() {
  return await path.resolve(await path.resourceDir(), 'static', 'icons', 'missing.png');
}

export async function updatePreviews(hwnds: HWND[]) {
  invoke(SeelenCommand.WegRequestUpdatePreviews, { handles: hwnds });
}

export async function iconPathFromExePath(exePath: string) {
  const parts = exePath.split('\\');
  const fileName = parts.at(-1)?.replace('.exe', '.png') || 'missing.png';
  return await path.resolve(await getGeneratedFilesPath(), 'icons', fileName);
}
