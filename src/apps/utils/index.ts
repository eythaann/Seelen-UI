import { path } from '@tauri-apps/api';
import { PhysicalSize } from '@tauri-apps/api/dpi';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

export function toPhysicalPixels(size: number): number {
  return Math.floor(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX() {
  let intallPath = await path.resourceDir();
  return intallPath.startsWith('C:\\Program Files\\WindowsApps');
}

export const setWindowAsFullSize = () => {
  const screenWidth = toPhysicalPixels(window.screen.width);
  const screenHeight = toPhysicalPixels(window.screen.height);
  getCurrent().setSize(new PhysicalSize(screenWidth, screenHeight));
};
