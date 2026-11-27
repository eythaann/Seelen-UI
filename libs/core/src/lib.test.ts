import { test } from "node:test";
import { ok as assert } from "node:assert/strict";
import * as SluLib from "./lib.ts";

test("Library is importable on background (no window)", () => {
  assert(!!SluLib);
});
