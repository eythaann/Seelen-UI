import { Rect } from '../../shared/app/Rect';

import { BorderState } from '../border/domain';
import { ContainerTabsState } from '../containerTopBar/domain';

export interface WMSettingsState {
  border: BorderState;
  autoStackinByCategory: boolean;
  resizeDelta: number;
  containerPadding: number;
  workspacePadding: number;
  globalWorkAreaOffset: Rect;
  containerTopBar: ContainerTabsState;
}