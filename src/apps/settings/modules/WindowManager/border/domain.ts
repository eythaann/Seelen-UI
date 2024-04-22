import { HexColor } from '../../shared/domain/interfaces';

export interface BorderState {
  enabled: boolean;
  offset: number;
  width: number;
  color: HexColor;
  activeColor: HexColor;
}