import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { PinnedWegItem, SwItemType } from 'seelen-core';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { ExtendedPinnedWegItem } from '../../shared/store/domain';

export function isWindowsLink(item: PinnedWegItem): boolean {
  return item.path.toLowerCase().endsWith('.lnk');
}

export class SwPinnedAppUtils {
  static async fromSaved(item: PinnedWegItem): Promise<ExtendedPinnedWegItem> {
    let icon_path =
      (await invoke<string | null>('get_icon', {
        path: isWindowsLink(item) ? item.path : item.execution_command,
      })) || LAZY_CONSTANTS.MISSING_ICON_PATH;

    return {
      ...item,
      type: SwItemType.Pinned,
      icon: convertFileSrc(icon_path),
      title: '',
      opens: [],
    };
  }
}
