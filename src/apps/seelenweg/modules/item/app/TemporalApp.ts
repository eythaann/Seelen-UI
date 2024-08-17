import { convertFileSrc } from '@tauri-apps/api/core';

import { fs } from '../../../../settings/modules/shared/tauri/infra';
import { getImageBase64FromUrl, LAZY_CONSTANTS } from '../../shared/utils/infra';

import { AppFromBackground, SpecialItemType, SwTemporalApp } from '../../shared/store/domain';

export class SwTemporalAppUtils {
  static async clean(item: AppFromBackground): Promise<AppFromBackground> {
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

  static fromBackground(item: AppFromBackground): SwTemporalApp {
    return {
      type: SpecialItemType.TemporalApp,
      icon: item.icon || '',
      exe: item.exe,
      execution_path: item.execution_path,
      title: item.exe.split('\\').at(-1) || 'Unknown',
      opens: [item.hwnd],
    };
  }
}
