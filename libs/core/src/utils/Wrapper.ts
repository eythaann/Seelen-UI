export type Wrapper = new <T>(plain: T) => T;

export const Wrapper = class Wrapper {
  public constructor(plain: unknown) {
    Object.assign(this, plain);
  }
} as Wrapper;
