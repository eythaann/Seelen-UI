/// <reference lib="deno.ns" />

import { build, type BuildOptions, emptyDir } from "@deno/dnt";

import denoJson from "../deno.json" with { type: "json" };

const { name, description, version, license } = denoJson;
const packageJson: BuildOptions["package"] = {
  name,
  description,
  version,
  license,
  repository: {
    type: "git",
    url: "git+https://github.com/eythaann/Seelen-UI.git",
  },
  bugs: {
    url: "https://github.com/eythaann/Seelen-UI/issues",
  },
};

await emptyDir("./npm"); // clear previous build
await build({
  compilerOptions: {
    lib: ["DOM", "DOM.Iterable", "ESNext"],
    target: "ES2023",
  },
  test: false, // this is performed by CI
  typeCheck: false, // this is performed by CI
  entryPoints: [
    {
      name: ".",
      path: "./mod.ts",
    },
    {
      name: "./types",
      path: "./gen/types/mod.ts",
    },
    {
      name: "./tauri",
      path: "./src/re-exports/tauri.ts",
    },
  ],
  outDir: "./npm",
  shims: {},
  importMap: "deno.json",
  package: packageJson,
  postBuild(): void {
    Deno.copyFileSync("../../LICENSE", "npm/LICENSE");
    Deno.copyFileSync("readme.md", "npm/readme.md");
    Deno.removeSync("npm/src", { recursive: true });

    // Inject the styles exports into the package.json
    const pkgPath = "npm/package.json";
    const pkg = JSON.parse(Deno.readTextFileSync(pkgPath));

    // Copy CSS assets — dnt only handles TS/JS, so we do this manually.
    Deno.mkdirSync("npm/styles", { recursive: true });
    for (const entry of Deno.readDirSync("styles")) {
      if (entry.isFile) {
        Deno.copyFileSync(`styles/${entry.name}`, `npm/styles/${entry.name}`);
      }
    }

    pkg.exports["./styles/*"] = "./styles/*";

    Deno.writeTextFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
  },
});
