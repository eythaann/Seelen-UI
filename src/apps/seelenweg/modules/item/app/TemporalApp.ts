import { WegItemType } from '@seelen-ui/lib';
import { convertFileSrc } from '@tauri-apps/api/core';

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
      type: WegItemType.Temporal,
      icon: item.icon || '',
      path: item.exe,
      execution_command: item.execution_path,
      is_dir: false,
      title: item.exe.split('\\').at(-1) || 'Unknown',
      opens: [{ hwnd: item.hwnd, presentative_monitor: item.presentative_monitor }],
    };
  }
}
