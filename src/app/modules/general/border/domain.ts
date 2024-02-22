import { HexColor } from '../../shared/domain/interfaces';

export interface BorderState {
  enable: boolean;
  offset: number;
  width: number;
  colorSingle: HexColor;
  colorMonocle: HexColor;
  colorStack: HexColor;
}