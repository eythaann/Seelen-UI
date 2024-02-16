import { AnimationsState } from '../animations/domain';
import { BorderState } from '../border/domain';

export interface GeneralSettingsState {
  border: BorderState;
  altFocusHack: boolean;
  autoStackinByCategory: boolean;
  animations: AnimationsState;
}