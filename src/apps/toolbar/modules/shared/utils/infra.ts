import { path } from '@tauri-apps/api';

export const LAZY_CONSTANTS = {
  MISSING_ICON_PATH: '',
  DEFAULT_THUMBNAIL: '',
};

export async function loadConstants() {
  let resourceDir = await path.resourceDir();

  LAZY_CONSTANTS.MISSING_ICON_PATH = await path.resolve(
    resourceDir,
    'static',
    'icons',
    'missing.png',
  );

  LAZY_CONSTANTS.DEFAULT_THUMBNAIL = await path.resolve(
    resourceDir,
    'static',
    'icons',
    'default_thumbnail.jpg',
  );
}
