import { SavedPinnedApp } from '../../../../shared/schemas/SeelenWegItems';
import { convertFileSrc } from '@tauri-apps/api/core';

import { getImageBase64FromUrl } from '../../shared/utils/infra';

import { SpecialItemType, SwPinnedApp } from '../../shared/store/domain';

export class SwPinnedAppUtils {
  static async fromSaved(item: SavedPinnedApp): Promise<SwPinnedApp> {
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
      title: '',
      opens: [],
    };
  }
}
