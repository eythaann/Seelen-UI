import { convertFileSrc } from '@tauri-apps/api/core';

import { fs } from '../../../../settings/modules/shared/tauri/infra';
import {
  getImageBase64FromUrl,
  getUWPInfoFromExePath,
  LAZY_CONSTANTS,
} from '../../shared/utils/infra';

import { filenameFromPath, getGeneratedFilesPath } from '../../shared/utils/app';

import { AppFromBackground, HWND, IApp, SpecialItemType } from '../../shared/store/domain';
import { UWP_IMAGE_POSTFIXES } from '../../shared/utils/domain';

export interface TemporalApp extends IApp {
  type: SpecialItemType.TemporalPin;
  opens: HWND[];
}

export class TemporalApp {
  static async cleanUWP(item: AppFromBackground) {
    try {
      const uwpPackage = await getUWPInfoFromExePath(item.exe);
      if (!uwpPackage) {
        return;
      }

      const app = uwpPackage.Applications.find(
        (app) => app.Executable.split('\\').at(-1)! === filenameFromPath(item.exe),
      );

      if (!app) {
        return;
      }

      item.execution_path = `shell:AppsFolder\\${uwpPackage.Name}_${uwpPackage.PublisherId}!${app.AppId}`;
      item.icon_path = await getGeneratedFilesPath() + '\\icons\\' + filenameFromPath(item.exe).replace('.exe', '_uwp.png');

      if (await fs.exists(item.icon_path)) {
        return;
      }

      for (const postfix of UWP_IMAGE_POSTFIXES) {
        const logoPathUWP =
          uwpPackage.InstallLocation + '\\' + app.Square44x44Logo.replace('.png', postfix);
        if (await fs.exists(logoPathUWP)) {
          await fs.copyFile(logoPathUWP, item.icon_path);
          // remove icon file generated from exe
          await fs.remove(item.icon_path.replace('_uwp.png', '.png'));
          break;
        }
      }
    } catch (error) {
      console.error('Error while getting UWP info: ', error);
    }
  }

  static async clean(item: AppFromBackground): Promise<AppFromBackground> {
    await TemporalApp.cleanUWP(item);

    if (!(await fs.exists(item.icon_path))) {
      item.icon_path = LAZY_CONSTANTS.MISSING_ICON_PATH;
    }

    try {
      item.icon = await getImageBase64FromUrl(convertFileSrc(item.icon_path));
    } catch {
      item.icon = convertFileSrc(item.icon_path);
    }
    return item;
  }

  static fromBackground(item: AppFromBackground): TemporalApp {
    return {
      type: SpecialItemType.TemporalPin,
      icon: item.icon || '',
      icon_path: item.icon_path,
      exe: item.exe,
      execution_path: item.execution_path,
      title: item.exe.split('\\').at(-1) || 'Unknown',
      opens: [item.hwnd],
    };
  }
}
