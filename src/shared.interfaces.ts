import { IUIColors } from '@seelen-ui/lib';
export interface IRootState<T> {
  settings: T;
  colors: IUIColors;
}
