import { Rect } from '../../shared/app/Rect';

import { BorderState } from '../border/domain';
import { ContainerTabsState } from '../containerTopBar/domain';
import { PopupState } from '../popups/domain';

export interface GeneralSettingsState {
  // own
  selectedTheme: string | null;
  // komorebi
  border: BorderState;
  popups: PopupState;
  autoStackinByCategory: boolean;
  resizeDelta: number;
  containerPadding: number;
  workspacePadding: number;
  globalWorkAreaOffset: Rect;
  containerTopBar: ContainerTabsState;
}