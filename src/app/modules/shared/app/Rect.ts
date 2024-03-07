import { ClassBuilder, Deserialize, Serialize } from 'readable-types';

export interface Rect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export const Rect = new ClassBuilder(
  class Rect {
    left: number = 0;
    top: number = 0;
    right: number = 0;
    bottom: number = 0;
  },
)
  .decorate(Serialize, Deserialize)
  .build();
