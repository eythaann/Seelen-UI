export enum ContainerTopBarMode {
  ON_STACK = 'OnStack',
  NEVER = 'Never',
}

export interface ContainerTabsState {
  mode: ContainerTopBarMode;
}