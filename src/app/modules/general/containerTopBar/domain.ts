import { HexColor } from '../../shared/domain/interfaces';

export enum ContainerTopBarMode {
  ALWAYS = 'Always',
  ON_STACK = 'OnStack',
  NEVER = 'Never',
}

export interface ContainerTabsState {
  mode: ContainerTopBarMode;
  height: number;
  tabs: {
    width: number;
    color: HexColor;
    background: HexColor;
  };
}