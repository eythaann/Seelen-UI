export enum SeelenWegMode {
  FULL_WIDTH = 'Full-Width',
  MIN_CONTENT = 'Min-Content',
}

export enum SeelenWegSide {
  LEFT = 'Left',
  RIGHT = 'Right',
  TOP = 'Top',
  BOTTOM = 'Bottom',
}

export interface SeelenWegState {
  enabled: boolean;
  mode: SeelenWegMode;
  position: SeelenWegSide;
  visibleSeparators: boolean;
  size: number;
  zoomSize: number;
  margin: number;
  padding: number;
  spaceBetweenItems: number;
}