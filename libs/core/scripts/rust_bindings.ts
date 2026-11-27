// Regenerates the TypeScript bindings (`./gen/types`) and JSON schemas (`./gen/schemas`) from the Rust types.
import { execSync } from "node:child_process";
import { appendFileSync, mkdirSync, readdirSync, rmSync } from "node:fs";

const GenTypesPath = "./gen/types";

console.log("[Task] Removing old bindings...");
rmSync(GenTypesPath, { recursive: true, force: true });
mkdirSync(GenTypesPath, { recursive: true });

console.log("[Task] Generating Typescript Bindings and JSON Schemas...");
// yeah cargo test generates the typescript bindings, why? ask to @aleph-alpha/ts-rs xd
// btw internally we also decided to use tests to avoid having a binary.
// also this will generate the json schemas
execSync("cargo test --features gen-binds", { stdio: "inherit" });

console.log("[Task] Creating entry points...");
for (const entry of readdirSync(GenTypesPath, { withFileTypes: true })) {
  if (entry.isFile() && entry.name.endsWith(".ts") && entry.name !== "mod.ts") {
    appendFileSync(`${GenTypesPath}/mod.ts`, `export * from './${entry.name}';\n`);
  }
}

console.log("[Task] Formatting...");
execSync("cargo fmt", { stdio: "inherit" });
execSync("npx --yes dprint fmt", { stdio: "inherit" });

console.log("[Task] Done!");
