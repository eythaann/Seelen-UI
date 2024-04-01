import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

import { store } from '../store/infra';

import { HWND, UWP } from '../store/domain';

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

export async function iconPathFromExePath(exePath: string) {
  const parts = exePath.split('\\');
  const fileName = parts.at(-1)?.replace('.exe', '.png') || 'missing.png';
  return await path.resolve(await path.resourceDir(), 'gen', 'icons', fileName);
}

export async function getUWPInfoFromExePath(exePath: string): Promise<UWP | undefined> {
  const dirname = await path.dirname(exePath);
  // for some reason uwp_manifests.json can no be readed and parsed by JSON.parse so
  // I use fetch as solution, maybe is a problem with the encoding of the file
  const response = await fetch(convertFileSrc(await path.resolveResource('gen/uwp_manifests.json')));
  const manifests: UWP[] = await response.json();
  return manifests.find((manifest) => manifest.InstallLocation === dirname);
}
