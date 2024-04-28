import { IRootState } from '../../../../../shared.interfaces';
import { FancyToolbar } from '../../../../utils/schemas/FancyToolbar';

export interface ActiveApp {
  title: string;
  name: string;
}

export interface RootState extends IRootState<FancyToolbar> {
  focused: ActiveApp | null;
}