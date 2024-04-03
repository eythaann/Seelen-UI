import { path } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/core';

import { fs } from '../../../../settings/modules/shared/infrastructure/tauri';
import { getImageBase64FromUrl, getUWPInfoFromExePath } from '../../shared/utils/infra';

import { HWND, IApp, SavedItems, SpecialItemType } from '../../shared/store/domain';

export interface PinnedApp extends IApp {
  type: SpecialItemType.PinnedApp;
  opens: HWND[];
}

export class PinnedApp {
  static async clean(item: SavedItems): Promise<SavedItems> {
    try {
      const uwpInfo = await getUWPInfoFromExePath(item.exe);
      if (uwpInfo && typeof uwpInfo.AppId === 'string') {
        item.execution_path = `shell:AppsFolder\\${uwpInfo.Name}_${uwpInfo.PublisherId}!${uwpInfo.AppId}`;
        const logoPath = uwpInfo.InstallLocation + '\\' + uwpInfo.Logo;
        const logoPath200 = uwpInfo.InstallLocation + '\\' + uwpInfo.Logo.replace('.png', '.scale-200.png');
        const logoPath400 = uwpInfo.InstallLocation + '\\' + uwpInfo.Logo.replace('.png', '.scale-400.png');

        if (await fs.exists(logoPath400)) {
          await fs.copyFile(logoPath400, item.icon_path);
        } else if (await fs.exists(logoPath200)) {
          await fs.copyFile(logoPath200, item.icon_path);
        } else if (await fs.exists(logoPath)) {
          await fs.copyFile(logoPath, item.icon_path);
        }
      }
    } catch (error) {
      console.error('Error while getting UWP info: ', error);
    }

    if (!(await fs.exists(item.icon_path))) {
      item.icon_path = await path.resolve(
        await path.resourceDir(),
        'static',
        'icons',
        'missing.png',
      );
    }

    try {
      item.icon = await getImageBase64FromUrl(convertFileSrc(item.icon_path));
    } catch {
      item.icon = convertFileSrc(item.icon_path);
    }

    return item;
  }

  static fromSaved(item: SavedItems): PinnedApp {
    return {
      type: SpecialItemType.PinnedApp,
      icon: item.icon || '',
      icon_path: item.icon_path,
      exe: item.exe,
      execution_path: item.execution_path,
      title: item.title,
      opens: [],
    };
  }
}