export type Enum<Literal extends string> =
  & { readonly [Key in Literal as Capitalize<Key>]: Key }
  // deno-lint-ignore ban-types
  & {};
