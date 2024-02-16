import { HexColor } from '../../shared/domain/interfaces';

export interface BorderState {
  enable: boolean;
  offset: null | number;
  width: number;
  color: HexColor;
}