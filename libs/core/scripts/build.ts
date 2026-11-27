// Builds the publishable JS package into `./dist`:
// - `dist/esm`: ES modules + type declarations (`import` condition)
// - `dist/script`: CommonJS + type declarations (`require` condition)
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, posix, relative, sep } from "node:path";

/** Import aliases (`tsconfig.json` paths) mapped to their emitted entry, relative to the output dir. */
const ALIASES: Record<string, string> = {
  "@seelen-ui/types": "gen/types/mod.js",
};

const require = createRequire(import.meta.url);
const tsc = require.resolve("typescript/bin/tsc");

function compile(outDir: string, module: string, moduleResolution: string, packageType: string): void {
  console.log(`[Task] Compiling ${outDir} (${module})...`);
  execFileSync(
    process.execPath,
    [tsc, "-p", "tsconfig.build.json", "--outDir", outDir, "--module", module, "--moduleResolution", moduleResolution],
    { stdio: "inherit" },
  );
  rewriteImports(outDir);
  // makes node resolve the right module format regardless of the root package.json `type`
  writeFileSync(`${outDir}/package.json`, JSON.stringify({ type: packageType }, null, 2) + "\n");
}

/**
 * tsc leaves two kinds of specifiers that consumers can't resolve:
 * - the `paths` aliases (tsc never rewrites them), in both JS and declarations.
 * - `.ts` extensions in declarations (`rewriteRelativeImportExtensions` only rewrites the JS).
 */
function rewriteImports(outDir: string): void {
  const specifier = /((?:from|import|require)\s*\(?\s*)(["'])([^"']+)\2/g;

  for (const file of readdirSync(outDir, { recursive: true, encoding: "utf8" })) {
    if (!file.endsWith(".js") && !file.endsWith(".d.ts")) continue;
    const path = `${outDir}/${file}`;
    const fileDir = dirname(file);
    const content = readFileSync(path, "utf8");

    const rewritten = content.replace(specifier, (match, prefix, quote, spec: string) => {
      const aliasTarget = ALIASES[spec];
      if (aliasTarget) {
        let rel = relative(fileDir, aliasTarget).split(sep).join(posix.sep);
        if (!rel.startsWith(".")) rel = `./${rel}`;
        return `${prefix}${quote}${rel}${quote}`;
      }
      if (spec.startsWith(".") && spec.endsWith(".ts") && !spec.endsWith(".d.ts")) {
        return `${prefix}${quote}${spec.slice(0, -3)}.js${quote}`;
      }
      return match;
    });

    if (rewritten !== content) writeFileSync(path, rewritten);
  }
}

rmSync("./dist", { recursive: true, force: true });
mkdirSync("./dist", { recursive: true });

compile("./dist/esm", "ESNext", "Bundler", "module");
compile("./dist/script", "CommonJS", "Node10", "commonjs");

// npm only packs files inside the package folder
copyFileSync("../../LICENSE", "./LICENSE");

console.log("[Task] Done!");
