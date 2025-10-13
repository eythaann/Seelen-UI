// Vanilla TypeScript build configuration

import esbuild from "esbuild";
import { createCopyPublicPlugin } from "../plugins/index.ts";
import type { BuildArgs } from "../types.ts";

/**
 * Creates esbuild configuration for Vanilla TypeScript applications
 */
export function createVanillaBuildConfig(
  entryPoints: string[],
  appFolders: string[],
  args: BuildArgs,
): esbuild.BuildOptions {
  return {
    entryPoints,
    bundle: true,
    minify: args.isProd,
    sourcemap: !args.isProd,
    treeShaking: true,
    format: "esm",
    outdir: "./dist",
    loader: {
      ".yml": "text",
      ".svg": "text",
    },
    plugins: [createCopyPublicPlugin(appFolders)],
  };
}

/**
 * Builds Vanilla TypeScript applications using esbuild
 */
export async function buildVanilla(
  entryPoints: string[],
  appFolders: string[],
  args: BuildArgs,
): Promise<void> {
  if (entryPoints.length === 0) {
    return;
  }

  const startTime = Date.now();
  const config = createVanillaBuildConfig(entryPoints, appFolders, args);

  if (args.serve) {
    const ctx = await esbuild.context(config);
    await ctx.watch();
    console.info(`✓ Vanilla: ${entryPoints.length} apps watching for changes`);
  } else {
    await esbuild.build(config);
    console.info(`✓ Vanilla: ${entryPoints.length} apps built (${Date.now() - startTime}ms)`);
  }
}
