import {
  SeelenLauncherWidgetId,
  SeelenToolbarWidgetId,
  SeelenWallWidgetId,
  SeelenWegWidgetId,
  SeelenWindowManagerWidgetId,
} from '@seelen-ui/lib';
import { Settings } from '@seelen-ui/lib/types';
import { cloneDeep, pick } from 'lodash';

import { RootState } from '../domain';

export const StateToJsonSettings = (state: RootState): Settings => {
  let settings = pick(cloneDeep(state), [
    'activeIconPacks',
    'oldActiveThemes',
    'activeThemes',
    'monitorsV3',
    'ahkEnabled',
    'ahkVariables',
    'devTools',
    'language',
    'dateFormat',
    'virtualDesktopStrategy',
    'updater',
    'byWidget',
    'drpc',
    'byTheme',
    'byWallpaper',
  ]);

  // migration since v2.1.0
  settings.byWidget[SeelenToolbarWidgetId] = state.fancyToolbar;
  settings.byWidget[SeelenLauncherWidgetId] = state.launcher;
  settings.byWidget[SeelenWallWidgetId] = state.wall;
  settings.byWidget[SeelenWegWidgetId] = state.seelenweg;
  settings.byWidget[SeelenWindowManagerWidgetId] = state.windowManager;

  return settings;
};
