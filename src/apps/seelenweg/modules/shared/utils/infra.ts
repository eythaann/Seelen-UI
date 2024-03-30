import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

import { store } from '../store/infra';

import { HWND } from '../store/domain';

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

export async function updatePreviews(hwnds: HWND[]) {
  const state = store.getState();
  const process = hwnds.map((hwnd) => {
    return state.openApps[hwnd]?.process_hwnd || 0;
  });
  invoke('weg_request_update_previews', { hwnds: process });
}
