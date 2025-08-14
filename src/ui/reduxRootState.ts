import { UIColors } from '@seelen-ui/lib/types';

export interface IRootState<T> {
  settings: T;
  colors: UIColors;
}
