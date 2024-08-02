import { SavedPinnedApp } from '../../../../shared/schemas/SeelenWegItems';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

import { getImageBase64FromUrl, LAZY_CONSTANTS } from '../../shared/utils/infra';

import { SpecialItemType, SwPinnedApp } from '../../shared/store/domain';

export class SwPinnedAppUtils {
  static async fromSaved(item: SavedPinnedApp): Promise<SwPinnedApp> {
    let icon = '';
    let icon_path = await invoke<string | null>('get_icon', { path: item.exe }) || LAZY_CONSTANTS.MISSING_ICON_PATH;

    try {
      icon = await getImageBase64FromUrl(convertFileSrc(icon_path));
    } catch {
      icon = convertFileSrc(icon_path);
    }

    return {
      type: SpecialItemType.PinnedApp,
      icon,
      exe: item.exe,
      execution_path: item.execution_path,
      title: '',
      opens: [],
    };
  }
}
