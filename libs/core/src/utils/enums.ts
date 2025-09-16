// deno-lint-ignore ban-types
export type Enum<Literal extends string> =
  & { readonly [Key in Literal as Capitalize<Key>]: Key }
  & {};
