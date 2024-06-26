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
  // TODO(eythan) this should be handle by the background process
  static async cleanUWP(item: AppFromBackground) {
    try {
      const uwpPackage = await getUWPInfoFromExePath(item.exe);
      if (!uwpPackage) {
        return;
      }

      const storedLogoPath =
        (await getGeneratedFilesPath()) +
        '\\icons\\' +
        filenameFromPath(item.exe).replace('.exe', '_uwp.png');

      const app = uwpPackage.Applications.find(
        (app) => app.Executable.split('\\').at(-1)! === filenameFromPath(item.exe),
      );

      if (app) {
        item.execution_path = `shell:AppsFolder\\${uwpPackage.Name}_${uwpPackage.PublisherId}!${app.AppId}`;
      }

      // check if a uwp logo already exists
      if (await fs.exists(storedLogoPath)) {
        return;
      }

      for (const postfix of UWP_IMAGE_POSTFIXES) {
        let logoToCopy = '';
        let storeLogo =
          uwpPackage.InstallLocation + '\\' + uwpPackage.StoreLogo.replace('.png', postfix);

        if (app) {
          logoToCopy =
            uwpPackage.InstallLocation + '\\' + app.Square150x150Logo.replace('.png', postfix);

          if (!(await fs.exists(logoToCopy))) {
            logoToCopy =
              uwpPackage.InstallLocation + '\\' + app.Square44x44Logo.replace('.png', postfix);

            if (!(await fs.exists(logoToCopy))) {
              logoToCopy = storeLogo;
            }
          }
        } else {
          logoToCopy = storeLogo;
        }

        if (!(await fs.exists(logoToCopy))) {
          continue;
        }

        item.icon_path = storedLogoPath;
        await fs.copyFile(logoToCopy, item.icon_path);
        // remove icon file generated from exe
        await fs.remove(item.icon_path.replace('_uwp.png', '.png'));
        break;
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
