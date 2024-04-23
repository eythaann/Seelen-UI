import { Rect } from '../../shared/app/Rect';

import { BorderState } from '../border/domain';
import { ContainerTabsState } from '../containerTopBar/domain';

interface FloatingWindowSettings {
  width: number;
  height: number;
}

export interface SeelenManagerState {
  border: BorderState;
  autoStackinByCategory: boolean;
  resizeDelta: number;
  containerPadding: number;
  workspacePadding: number;
  globalWorkAreaOffset: Rect;
  containerTopBar: ContainerTabsState;
  floating: FloatingWindowSettings;
}