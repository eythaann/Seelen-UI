export enum SeelenWegMode {
  FULL_WIDTH = 'Full-Width',
  MIN_CONTENT = 'Min-Content',
}

export interface SeelenWegState {
  enabled: boolean;
  mode: SeelenWegMode;
  size: number;
  zoomSize: number;
  margin: number;
  padding: number;
  spaceBetweenItems: number;
}