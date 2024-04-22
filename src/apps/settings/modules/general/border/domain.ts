import { HexColor } from '../../shared/domain/interfaces';

export interface BorderState {
  enable: boolean;
  offset: number;
  width: number;
  color: HexColor;
  activeColor: HexColor;
}