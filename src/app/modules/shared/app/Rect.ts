
export namespace Rect {
  export type plain = Omit<Rect, 'plain'>;
}

export interface Rect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export class Rect {
  left: number = 0;
  top: number = 0;
  right: number = 0;
  bottom: number = 0;

  plain() {
    const { ...obj } = this;
    return obj;
  }
}