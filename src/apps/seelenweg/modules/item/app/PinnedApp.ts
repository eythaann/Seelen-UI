import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { PinnedWegItem, SwItemType } from 'seelen-core';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { ExtendedPinnedWegItem } from '../../shared/store/domain';

export class SwPinnedAppUtils {
  static async fromSaved(item: PinnedWegItem): Promise<ExtendedPinnedWegItem> {
    let icon_path =
      (await invoke<string | null>('get_icon', { path: item.exe })) ||
      LAZY_CONSTANTS.MISSING_ICON_PATH;

    return {
      type: SwItemType.PinnedApp,
      icon: convertFileSrc(icon_path),
      exe: item.exe,
      execution_path: item.execution_path,
      title: '',
      opens: [],
    };
  }
}
