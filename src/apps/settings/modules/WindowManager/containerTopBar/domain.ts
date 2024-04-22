export enum ContainerTopBarMode {
  ALWAYS = 'Always',
  ON_STACK = 'OnStack',
  NEVER = 'Never',
}

export interface ContainerTabsState {
  mode: ContainerTopBarMode;
}