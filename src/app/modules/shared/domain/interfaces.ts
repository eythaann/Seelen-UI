export type HexColor = `#${string}`;

export type SelectorsFor<T> = { [K in keyof T]: (state: T) => T[K] };