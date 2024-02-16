export type HexColor = `#${string}`;

export type PickLessObject<T> = { [K in keyof T as T[K] extends object ? never : K]: T[K] };