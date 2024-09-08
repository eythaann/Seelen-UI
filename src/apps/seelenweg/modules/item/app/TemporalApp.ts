import { convertFileSrc } from '@tauri-apps/api/core';
import { SwItemType } from 'seelen-core';

import { fs } from '../../../../settings/modules/shared/tauri/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { AppFromBackground, ExtendedTemporalWegItem } from '../../shared/store/domain';

export class SwTemporalAppUtils {
  static async clean(item: AppFromBackground): Promise<AppFromBackground> {
    if (!(await fs.exists(item.icon_path))) {
      item.icon_path = LAZY_CONSTANTS.MISSING_ICON_PATH;
    }
    item.icon = convertFileSrc(item.icon_path);
    return item;
  }

  static fromBackground(item: AppFromBackground): ExtendedTemporalWegItem {
    return {
      type: SwItemType.TemporalApp,
      icon: item.icon || '',
      exe: item.exe,
      execution_path: item.execution_path,
      title: item.exe.split('\\').at(-1) || 'Unknown',
      opens: [item.hwnd],
    };
  }
}
