import { convertFileSrc } from '@tauri-apps/api/core';

import { fs } from '../../../../settings/modules/shared/infrastructure/tauri';
import { getImageBase64FromUrl, LAZY_CONSTANTS } from '../../shared/utils/infra';

import { HWND, IApp, SavedAppsInYaml, SpecialItemType } from '../../shared/store/domain';

export interface PinnedApp extends IApp {
  type: SpecialItemType.PinnedApp;
  opens: HWND[];
}

export class PinnedApp {
  static async clean(item: SavedAppsInYaml): Promise<SavedAppsInYaml> {
    if (!(await fs.exists(item.icon_path))) {
      item.icon_path = LAZY_CONSTANTS.MISSING_ICON_PATH;
    }
    return item;
  }

  static async fromSaved(item: SavedAppsInYaml): Promise<PinnedApp> {
    let icon = '';

    try {
      icon = await getImageBase64FromUrl(convertFileSrc(item.icon_path));
    } catch {
      icon = convertFileSrc(item.icon_path);
    }

    return {
      type: SpecialItemType.PinnedApp,
      icon,
      icon_path: item.icon_path,
      exe: item.exe,
      execution_path: item.execution_path,
      title: item.title,
      opens: [],
    };
  }
}