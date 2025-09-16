import { assert } from "@std/assert/assert";
import * as SluLib from "./lib.ts";

Deno.test("Library is importable on background (no window)", () => {
  assert(!!SluLib);
});
